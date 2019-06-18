// This code is taken from the flamegraph.rs from rbspy
// https://github.com/rbspy/rbspy/tree/master/src/ui/flamegraph.rs
// licensed under the MIT License:
/*
MIT License

Copyright (c) 2016 Julia Evans, Kamal Marhubi
Portions (continuous integration setup) Copyright (c) 2016 Jorge Aparicio

Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all
copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
SOFTWARE.
*/

use std::collections::HashMap;
use std::collections::BTreeMap;
use std::ops::Bound::Included;
use std::ops::Bound::Excluded;
use std::fs::File;


use failure::Error;
use inferno::flamegraph::{Direction, Options};

use stack_trace::StackTrace;
use serde::{Deserialize, Serialize};

type Records = HashMap<String, BTreeMap<u64, usize>>;

#[derive(Serialize, Deserialize)]
pub struct Flamegraph {
    pub counts: Records,
    pub show_linenumbers: bool,
}

impl Flamegraph {
    pub fn new(show_linenumbers: bool) -> Flamegraph {
        Flamegraph { counts: HashMap::new(), show_linenumbers }
    }

    pub fn increment(&mut self, time_stamp: u64, traces: &[StackTrace]) -> std::io::Result<()> {
        for trace in traces {
            if !(trace.active) {
                continue;
            }

            // convert the frame into a single ';' delimited String
            let frame = trace.frames.iter().rev().map(|frame| {
                let filename = match &frame.short_filename { Some(f) => &f, None => &frame.filename };
                if self.show_linenumbers && frame.line != 0 {
                    format!("{} ({}:{})", frame.name, filename, frame.line)
                } else {
                    format!("{} ({})", frame.name, filename)
                }
            }).collect::<Vec<String>>().join(";");

            // update counts for that frame
            let mut statistics = self.counts.entry(frame).or_insert(BTreeMap::new());
            *statistics.entry(time_stamp).or_insert(0) += 1;
        }
        Ok(())
    }

    pub fn write_file(&self, w: File, start_ts: u64, end_ts: u64) -> Result<(), Error> {
        let records = self.filter_records(start_ts, end_ts);
        let lines: Vec<String> = records.iter().map(|(k, v)| format!("{} {}", k, v)).collect();
        let mut opts =  Options {
            direction: Direction::Inverted,
            min_width: 1.0,
            title: "py-spy".to_owned(),
            ..Default::default()
        };

        inferno::flamegraph::from_lines(&mut opts, lines.iter().map(|x| x.as_str()), w).unwrap();
        Ok(())
    }

    fn filter_records(&self, start_ts: u64, end_ts: u64) -> HashMap<String, usize> {
        let mut ret = HashMap::new();
        for (stack_str, statistics) in &self.counts {
            let mut counter: usize = 0;
            for (_, ref num) in statistics.range((Included(&start_ts), Excluded(&end_ts))) {
                counter += **num;
            }

            if counter > 0 {
                ret.insert(stack_str.clone(), counter);
            }
        }
        ret
    }
}

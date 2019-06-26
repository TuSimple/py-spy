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

    pub fn increment(&mut self, time_stamp: u64, traces: &StackTrace) -> std::io::Result<()> {
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
        let statistics = self.counts.entry(frame).or_insert(BTreeMap::new());
        *statistics.entry(time_stamp).or_insert(0) += 1;
        Ok(())
    }

    fn get_lines(&self) -> Vec<String> {
        self.counts.iter().map(|(k, v)| format!("{} {}", k, v)).collect()
    }

    pub fn write(&self, w: File, start_ts: u64, end_ts: u64) -> Result<(), Error> {
        let records = self.filter_records(start_ts, end_ts);
        let lines: Vec<String> = records.iter().map(|(k, v)| format!("{} {}", k, v)).collect();
        let mut opts =  Options {
            direction: Direction::Inverted,
            min_width: 1.0,
            title: "py-spy".to_owned(),
            ..Default::default()
        };

        let lines = self.get_lines();
        inferno::flamegraph::from_lines(&mut opts, lines.iter().map(|x| x.as_str()), w)
            .map_err(|e| format_err!("Failed to write flamegraph: {}", e))?;
        Ok(())
    }

    pub fn write_raw(&self, w: &mut File) -> Result<(), Error> {
        use std::io::Write;
        for line in self.get_lines() {
            w.write_all(line.as_bytes())?;
            w.write_all(b"\n")?;
        }
        Ok(())
    }

    pub fn filter_records(&self, start_ts: u64, end_ts: u64) -> HashMap<String, usize> {
        let mut ret = HashMap::new();
        if start_ts < end_ts {
            for (stack_str, statistics) in &self.counts {
                let mut counter: usize = 0;
                for (_, ref num) in statistics.range((Included(&start_ts), Excluded(&end_ts))) {
                    counter += **num;
                }

                if counter > 0 {
                    ret.insert(stack_str.clone(), counter);
                }
            }
        } else {
            eprintln!("Error: Invalid time interval [{}, {})", start_ts, end_ts);
        }
        ret
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_invalid_time_interval() {
        let mut test_flame = Flamegraph::new(true);
        let stack_trace_a = "test_stack_trace_a";
        let stack_trace_b = "test_stack_trace_b";

        {
            let test_records = &mut test_flame.counts;
            
            // Insert two records
            (*test_records.entry(String::from(stack_trace_a)).or_insert(BTreeMap::new())).insert(5, 2);
            (*test_records.entry(String::from(stack_trace_b)).or_insert(BTreeMap::new())).insert(10, 2); 
        }

        let test_ret = test_flame.filter_records(5, 0);
        assert!(test_ret.is_empty());

        let test_ret = test_flame.filter_records(5, 5);
        assert!(test_ret.is_empty());

        let test_ret = test_flame.filter_records(5, 6);
        let stack_trace_a = String::from(stack_trace_a);
        assert_eq!(test_ret.len(), 1);
        assert!(test_ret.contains_key(&stack_trace_a));
        assert_eq!(*test_ret.get(&stack_trace_a).unwrap_or(&0), 2);

        let test_ret = test_flame.filter_records(10, 0);
        assert!(test_ret.is_empty());

        let test_ret = test_flame.filter_records(10, 10);
        assert!(test_ret.is_empty());

        let test_ret = test_flame.filter_records(10, 11);
        let stack_trace_b = String::from(stack_trace_b);
        assert_eq!(test_ret.len(), 1);
        assert!(test_ret.contains_key(&stack_trace_b));
        assert_eq!(*test_ret.get(&stack_trace_b).unwrap_or(&0), 2);
    }

    #[test]
    fn test_single_value_filtering() {
        let mut test_flame = Flamegraph::new(true);
        let stack_trace_a = "test_stack_trace_a";
        let stack_trace_b = "test_stack_trace_b";

        {
            let test_records = &mut test_flame.counts;
            
            // Insert two records
            (*test_records.entry(String::from(stack_trace_a)).or_insert(BTreeMap::new())).insert(5, 3);
            (*test_records.entry(String::from(stack_trace_b)).or_insert(BTreeMap::new())).insert(10, 3); 
        }

        let test_ret = test_flame.filter_records(5, 10);
        let stack_trace_a = String::from(stack_trace_a);
        assert_eq!(test_ret.len(), 1);
        assert!(test_ret.contains_key(&stack_trace_a));
        assert_eq!(*test_ret.get(&stack_trace_a).unwrap_or(&0), 3);

        let test_ret = test_flame.filter_records(10, 11);
        let stack_trace_b = String::from(stack_trace_b);
        assert_eq!(test_ret.len(), 1);
        assert!(test_ret.contains_key(&stack_trace_b));
        assert_eq!(*test_ret.get(&stack_trace_b).unwrap_or(&0), 3);

        let test_ret = test_flame.filter_records(5, 11);
        assert_eq!(test_ret.len(), 2);
        assert!(test_ret.contains_key(&stack_trace_a));
        assert_eq!(*test_ret.get(&stack_trace_a).unwrap_or(&0), 3);
        assert!(test_ret.contains_key(&stack_trace_b));
        assert_eq!(*test_ret.get(&stack_trace_b).unwrap_or(&0), 3);
    }

    #[test]
    fn test_multiple_values_aggregation() {
        let mut test_flame = Flamegraph::new(true);
        let stack_trace_a = "test_stack_trace_a";
        let stack_trace_b = "test_stack_trace_b";

        // Insert records into stack_trace_a
        {
            let test_records = &mut test_flame.counts;
            test_records.insert(String::from(stack_trace_a), BTreeMap::new());

            let stack_trace_a_string = String::from(stack_trace_a);
            let statistics = test_records.get_mut(&stack_trace_a_string).unwrap();

            // Insert records
            statistics.insert(5, 1);
            statistics.insert(10, 2);
            statistics.insert(15, 3);
        }

        // Insert records into stack_trace_b
        {
            let test_records = &mut test_flame.counts;
            test_records.insert(String::from(stack_trace_b), BTreeMap::new());

            let stack_trace_b_string = String::from(stack_trace_b);
            let statistics = test_records.get_mut(&stack_trace_b_string).unwrap();

            // Insert records
            statistics.insert(7, 4);
            statistics.insert(12, 5);
            statistics.insert(19, 6);
        }

        let stack_trace_a = String::from(stack_trace_a);
        let stack_trace_b = String::from(stack_trace_b);

        let test_ret = test_flame.filter_records(5, 21);
        assert_eq!(test_ret.len(), 2);
        assert!(test_ret.contains_key(&stack_trace_a));
        assert!(test_ret.contains_key(&stack_trace_b));
        assert_eq!(*test_ret.get(&stack_trace_a).unwrap_or(&0), 6);
        assert_eq!(*test_ret.get(&stack_trace_b).unwrap_or(&0), 15);
    }
}

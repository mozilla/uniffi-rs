/// Generic code for handling code generation passes

use crate::log::Logger;
use std::fmt::Debug;
use std::collections::HashSet;
use std::hash::Hash;

/// `Pass` performs a single code-generation pass that transforms an input Vec into an output Vec
pub trait Pass<T> {
    type OutputType;

    fn run_pass(&self, input: Vec<T>) -> Vec<Self::OutputType>;
}

/// `VecPass` defines the run_pass method on Vec which is convenient because then you can chain
/// passes together.
pub trait VecPass<T>: Sized {
    fn run_pass<P: Pass<T>>(self, pass: &P) -> Vec<P::OutputType>;
}

impl<T> VecPass<T> for Vec<T> {
    fn run_pass<P: Pass<T>>(self, pass: &P) -> Vec<P::OutputType> {
        pass.run_pass(self)
    }
}

/// Pass that maps each input element to N output elements
pub struct Map<I, O, L> where
    I: Debug,
    O: Debug + Eq + Hash + Clone,
    L: Logger
{
    pub map_func: Box<dyn Fn(I) -> Vec<O>>,
    pub logger: L,
    pub prevent_dupes: bool,
}

impl<I, O, L> Map<I, O, L> where
    I: Debug,
    O: Debug + Eq + Hash + Clone,
    L: Logger
{
    fn map_one_item(&self, i: usize, item: I, seen: &mut HashSet<O>) -> Vec<O> {
        if i > 0 {
            self.logger.log_separator();
        }
        self.logger.log_input(&item);
        let result = (self.map_func)(item);
        result.into_iter().filter(|i| {
            if !(self.prevent_dupes && !seen.insert(i.clone())) {
                self.logger.log_output(i);
                true
            } else {
                self.logger.log_dupe(i);
                false
            }
        }).collect()
    }
}


impl<I, O, L> Pass<I> for Map<I, O, L> where
    I: Debug,
    O: Debug + Eq + Hash + Clone,
    L: Logger
{
    type OutputType = O;

    fn run_pass(&self, input: Vec<I>) -> Vec<O> {
        let mut seen: HashSet<O> = HashSet::new();
        input.into_iter().enumerate().map(|(i, item)| self.map_one_item(i, item, &mut seen)).flatten().collect()
    }
}

/// Pass that runs an arbitrary transform
pub struct Transform<I, O, L> where
    I: Debug,
    O: Debug,
    L: Logger
{
    pub map_func: Box<dyn Fn(Vec<I>) -> Vec<O>>,
    pub logger: L,
}

impl<I, O, L> Pass<I> for Transform<I, O, L> where
    I: Debug,
    O: Debug,
    L: Logger
{
    type OutputType = O;

    fn run_pass(&self, input: Vec<I>) -> Vec<O> {
        input.iter().for_each(|i| self.logger.log_input(i));
        self.logger.log_separator();
        let output = (self.map_func)(input);
        output.iter().for_each(|i| self.logger.log_output(i));
        output
    }
}


#[cfg(test)]
mod test {
    use super::*;
    use crate::log::StringLogger;

    fn get_logger_lines(logger: &StringLogger) -> Vec<String> {
        let mut lines: Vec<String> = logger.value().split("\n").map(|s| s.into()).collect();
        lines.pop(); // drop the last line of split(), which is always empty
        lines
    }

    // Low level run_pass calls
    struct ConvertStringToInt;

    impl Pass<&str> for ConvertStringToInt {
        type OutputType = i32;
        fn run_pass(&self, input: Vec<&str>) -> Vec<i32> {
            input.into_iter().map(|s| s.parse().unwrap()).collect()
        }
    }

    struct Square;

    impl Pass<i32> for Square {
        type OutputType = i32;
        fn run_pass(&self, input: Vec<i32>) -> Vec<i32> {
            input.into_iter().map(|i| i * i).collect()
        }
    }

    #[test]
    fn test_pass() {
        assert_eq!(
            ConvertStringToInt.run_pass(vec!["1", "2", "3"]),
            vec![1, 2, 3]
        );
    }

    #[test]
    fn test_chain_pass() {
        let input = vec!["1", "2", "3"];
        assert_eq!(
            input.run_pass(&ConvertStringToInt).run_pass(&Square),
            vec![1, 4, 9]
        );
    }

    // Map
    #[test]
    fn test_map() {
        let duplicate = Map {
            map_func: Box::new(|i| vec![i, i]),
            logger: StringLogger::new(),
            prevent_dupes: false,
        };
        assert_eq!(
            duplicate.run_pass(vec![1, 2, 3]),
            vec![1, 1, 2, 2, 3, 3]
        );
        assert_eq!(
            get_logger_lines(&duplicate.logger),
            vec![
                "< 1",
                "> 1",
                "> 1",
                "",
                "< 2",
                "> 2",
                "> 2",
                "",
                "< 3",
                "> 3",
                "> 3",
            ]
        )
    }

    #[test]
    fn test_prevent_dupes() {
        let mod3 = Map {
            map_func: Box::new(|i| vec![i % 3]),
            logger: StringLogger::new(),
            prevent_dupes: true,
        };
        assert_eq!(
            mod3.run_pass(vec![1, 2, 4, 6]),
            vec![1, 2, 0],
        );
        assert_eq!(
            get_logger_lines(&mod3.logger),
            vec![
                "< 1",
                "> 1",
                "",
                "< 2",
                "> 2",
                "",
                "< 4",
                "X 1",
                "",
                "< 6",
                "> 0",
            ]
        )
    }

    // Transform

    #[test]
    fn test_transform() {
        let reverse = Transform {
            map_func: Box::new(|input| input.into_iter().rev().collect()),
            logger: StringLogger::new(),
        };
        assert_eq!(
            reverse.run_pass(vec![1, 2, 3]),
            vec![3, 2, 1]
        );
        assert_eq!(
            get_logger_lines(&reverse.logger),
            vec![
                "< 1",
                "< 2",
                "< 3",
                "",
                "> 3",
                "> 2",
                "> 1",
            ]
        );
    }
}

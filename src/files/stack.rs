#![allow(unused_variables, unused_mut, dead_code, non_snake_case, unused_imports, unreachable_patterns)]
use std::io::{self, BufRead, Write};

#[derive(Debug, Clone, Default)]
pub struct Stack {
    pub data: Option<Vec<i64>>,
    pub top: Option<i64>,
}

#[derive(Debug, Clone, Default)]
pub struct Queue {
    pub data: Option<Vec<i64>>,
    pub head: Option<i64>,
    pub tail: Option<i64>,
}

pub fn make_stack() -> Option<Box<Stack>> {
    let mut s: Option<Box<Stack>> = None;
    s.as_mut().unwrap().data = Some([]);
    s.as_mut().unwrap().top = Some((-1_i64));
    return s;
}

pub fn push(mut s: Option<Box<Stack>>, mut val: i64) {
    s.as_ref().unwrap().data.unwrap().push(val);
    s.as_mut().unwrap().top = Some((s.as_ref().unwrap().top.unwrap() + 1_i64));
}

pub fn stack_pop(mut s: Option<Box<Stack>>) -> i64 {
    if (s.as_ref().unwrap().top.unwrap() == (-1_i64)) {
        return (-1_i64);
    }
    let mut val: i64 = s.as_ref().unwrap().data[s.as_ref().unwrap().top.unwrap() as usize];
    s.as_mut().unwrap().top = Some((s.as_ref().unwrap().top.unwrap() - 1_i64));
    return val;
}

pub fn stack_empty(mut s: Option<Box<Stack>>) -> bool {
    return (s.as_ref().unwrap().top.unwrap() == (-1_i64));
}

pub fn make_queue() -> Option<Box<Queue>> {
    let mut q: Option<Box<Queue>> = None;
    q.as_mut().unwrap().data = Some([]);
    q.as_mut().unwrap().head = Some(0_i64);
    q.as_mut().unwrap().tail = Some(0_i64);
    return q;
}

pub fn enqueue(mut q: Option<Box<Queue>>, mut val: i64) {
    q.as_ref().unwrap().data.unwrap().push(val);
    q.as_mut().unwrap().tail = Some((q.as_ref().unwrap().tail.unwrap() + 1_i64));
}

pub fn dequeue(mut q: Option<Box<Queue>>) -> i64 {
    if (q.as_ref().unwrap().head.unwrap() == q.as_ref().unwrap().tail.unwrap()) {
        return (-1_i64);
    }
    let mut val: i64 = q.as_ref().unwrap().data[q.as_ref().unwrap().head.unwrap() as usize];
    q.as_mut().unwrap().head = Some((q.as_ref().unwrap().head.unwrap() + 1_i64));
    return val;
}

pub fn queue_empty(mut q: Option<Box<Queue>>) -> bool {
    return (q.as_ref().unwrap().head.unwrap() == q.as_ref().unwrap().tail.unwrap());
}

pub fn sort_with_stack(mut lst: Vec<i64>) -> Vec<i64> {
    let mut s: Option<Box<Stack>> = make_stack();
    let mut result: Vec<i64> = [];
    {
        let mut i: i64 = 0_i64;
        while i < (lst.len() as i64) {
            let mut val: i64 = lst[i as usize];
            {
                let mut j: i64 = 0_i64;
                while j < 1000_i64 {
                    if stack_empty(s) {
                        break;
                    }
                    if (s.as_ref().unwrap().data[s.as_ref().unwrap().top.unwrap() as usize] <= val) {
                        break;
                    }
                    result.push(stack_pop(s));
                    j += 1_i64;
                }
            }
            push(s, val);
            i += 1_i64;
        }
    }
    {
        let mut i: i64 = 0_i64;
        while i < 1000_i64 {
            if stack_empty(s) {
                break;
            }
            result.push(stack_pop(s));
            i += 1_i64;
        }
    }
    return result;
}

fn main() {
    let mut stk: Option<Box<Stack>> = make_stack();
    push(stk, 10_i64);
    push(stk, 3_i64);
    push(stk, 7_i64);
    push(stk, 1_i64);
    let mut q: Option<Box<Queue>> = make_queue();
    {
        let mut i: i64 = 0_i64;
        while i < 4_i64 {
            let mut v: i64 = stack_pop(stk);
            enqueue(q, v);
            println!("{} {}", "popped and enqueued: {}".to_string(), v);
            i += 1_i64;
        }
    }
    {
        let mut i: i64 = 0_i64;
        while i < 4_i64 {
            let mut v: i64 = dequeue(q);
            println!("{} {}", "dequeued: {}".to_string(), v);
            i += 1_i64;
        }
    }
    let mut unsorted: Vec<i64> = [5_i64, 2_i64, 8_i64, 1_i64, 9_i64, 3_i64];
    let mut sorted: Vec<i64> = sort_with_stack(unsorted);
    {
        let mut i: i64 = 0_i64;
        while i < (sorted.len() as i64) {
            println!("{} {}", "{}".to_string(), sorted[i as usize]);
            i += 1_i64;
        }
    }
}

use std::collections::{BTreeMap, VecDeque};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum StableIdError {
    EmptyGraph,
    UnknownSuccessor,
}

pub fn breadth_first_order(
    entry: usize,
    block_count: usize,
    mut successors: impl FnMut(usize) -> Vec<usize>,
) -> Result<Vec<usize>, StableIdError> {
    if block_count == 0 || entry >= block_count {
        return Err(StableIdError::EmptyGraph);
    }
    let mut discovered = vec![false; block_count];
    let mut queue = VecDeque::from([entry]);
    let mut order = Vec::new();
    discovered[entry] = true;
    while let Some(block) = queue.pop_front() {
        order.push(block);
        for successor in successors(block) {
            if successor >= block_count {
                return Err(StableIdError::UnknownSuccessor);
            }
            if !discovered[successor] {
                discovered[successor] = true;
                queue.push_back(successor);
            }
        }
    }
    Ok(order)
}

pub fn block_names(order: &[usize]) -> BTreeMap<usize, String> {
    order
        .iter()
        .enumerate()
        .map(|(index, block)| (*block, format!("bb{index}")))
        .collect()
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct DenseIds {
    next_temporary: usize,
    next_parameter: usize,
}

impl DenseIds {
    pub fn temporary(&mut self) -> String {
        let id = format!("t{}", self.next_temporary);
        self.next_temporary += 1;
        id
    }

    pub fn block_parameter(&mut self) -> String {
        let id = format!("p{}", self.next_parameter);
        self.next_parameter += 1;
        id
    }
}

use std::cell::UnsafeCell;
use std::collections::HashSet;
use typed_arena::Arena;

struct Node<'a> {
    edges: UnsafeCell<Vec<&'a Node<'a>>>,
}

impl<'a> Node<'a> {
    fn new<'b>(arena: &'b Arena<Node<'b>>) -> &'b Node<'b> {
        arena.alloc(Node {
            edges: UnsafeCell::new(Vec::new())
        })
    }

    fn traverse<F>(&self, f: &F, seen: &mut HashSet<&'static str>)
        where F: Fn(&'static str)
    {
        unsafe {
            for n in &(*self.edges.get()) {
                n.traverse(f, seen);
            }
        }
    }

    fn first(&'a self) -> &'a Node<'a> {
        unsafe {
            (*self.edges.get())[0]
        }
    }
}

fn init<'a>(arena: &'a Arena<Node<'a>>) ->&'a Node<'a> {
    let root = Node::new(arena);

    let b = Node::new(arena);
    let c = Node::new(arena);
    let d = Node::new(arena);
    let e = Node::new(arena);
    let f = Node::new(arena);

    unsafe {
        (*root.edges.get()).push(b);
        (*root.edges.get()).push(c);
        (*root.edges.get()).push(d);

        (*c.edges.get()).push(e);
        (*c.edges.get()).push(f);
        (*c.edges.get()).push(root);
    }

    root
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn initialize_graph() {

    }

}


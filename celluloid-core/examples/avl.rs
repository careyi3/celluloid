//! cargo run -p celluloid-core --example avl
//! then: celluloid avl.json
//!
//! Each insert is shown twice: first as a plain BST insert, then after
//! rebalancing, so rotations animate as nodes swinging into place.

use celluloid_core::{Recorder, Tree};

#[derive(Default)]
struct Avl {
    key: Vec<i32>,
    left: Vec<Option<usize>>,
    right: Vec<Option<usize>>,
    height: Vec<i32>,
    root: Option<usize>,
}

impl Avl {
    fn h(&self, n: Option<usize>) -> i32 {
        n.map_or(0, |n| self.height[n])
    }

    fn update(&mut self, n: usize) {
        self.height[n] = 1 + self.h(self.left[n]).max(self.h(self.right[n]));
    }

    fn balance(&self, n: usize) -> i32 {
        self.h(self.left[n]) - self.h(self.right[n])
    }

    fn rotate_right(&mut self, y: usize) -> usize {
        let x = self.left[y].unwrap();
        self.left[y] = self.right[x];
        self.right[x] = Some(y);
        self.update(y);
        self.update(x);
        x
    }

    fn rotate_left(&mut self, x: usize) -> usize {
        let y = self.right[x].unwrap();
        self.right[x] = self.left[y];
        self.left[y] = Some(x);
        self.update(x);
        self.update(y);
        y
    }

    fn insert(&mut self, key: i32) -> usize {
        let n = self.key.len();
        self.key.push(key);
        self.left.push(None);
        self.right.push(None);
        self.height.push(1);
        let mut at = match self.root {
            None => {
                self.root = Some(n);
                return n;
            }
            Some(r) => r,
        };
        loop {
            let slot = if key < self.key[at] {
                &mut self.left[at]
            } else {
                &mut self.right[at]
            };
            match *slot {
                Some(next) => at = next,
                None => {
                    *slot = Some(n);
                    return n;
                }
            }
        }
    }

    /// Rebalance every node bottom-up, returning the nodes rotated around.
    fn rebalance(&mut self, n: Option<usize>, pivots: &mut Vec<usize>) -> Option<usize> {
        let n = n?;
        self.left[n] = self.rebalance(self.left[n], pivots);
        self.right[n] = self.rebalance(self.right[n], pivots);
        self.update(n);
        let b = self.balance(n);
        Some(if b > 1 {
            pivots.push(n);
            if self.balance(self.left[n].unwrap()) < 0 {
                self.left[n] = Some(self.rotate_left(self.left[n].unwrap()));
            }
            self.rotate_right(n)
        } else if b < -1 {
            pivots.push(n);
            if self.balance(self.right[n].unwrap()) > 0 {
                self.right[n] = Some(self.rotate_right(self.right[n].unwrap()));
            }
            self.rotate_left(n)
        } else {
            n
        })
    }

    /// Mirror every link into the recording; unchanged links are dropped.
    fn sync(&self, rec: &mut Recorder, t: Tree) {
        for n in 0..self.key.len() {
            match self.left[n] {
                Some(l) => rec.left(t, self.key[n], self.key[l]),
                None => rec.clear_child(t, self.key[n], 0),
            }
            match self.right[n] {
                Some(r) => rec.right(t, self.key[n], self.key[r]),
                None => rec.clear_child(t, self.key[n], 1),
            }
            rec.label(t, self.key[n], format!("bf {}", self.balance(n)));
        }
    }
}

pub fn record() -> Recorder {
    let mut rec = Recorder::new("AVL tree");
    rec.frame_delay(500.0);
    let t = rec.tree("avl");
    rec.state(t, "new", "#f28e2b");
    rec.state(t, "pivot", "#e15759");

    let mut avl = Avl::default();
    for key in [50, 30, 70, 20, 10, 60, 80, 65, 67, 90, 95, 5, 1, 66] {
        rec.fill(t, "default");
        let n = avl.insert(key);
        rec.node(t, key);
        rec.set(t, avl.key[n], "new");
        rec.marker(t, "inserted", avl.key[n]);
        avl.sync(&mut rec, t);
        rec.var("size", avl.key.len());
        rec.frame(format!("insert {key}"));

        let mut pivots = Vec::new();
        avl.root = avl.rebalance(avl.root, &mut pivots);
        if !pivots.is_empty() {
            for &p in &pivots {
                rec.set(t, avl.key[p], "pivot");
            }
            avl.sync(&mut rec, t);
            rec.bookmark(format!("rotate around {}", avl.key[pivots[0]]));
            rec.frame(format!("rebalance around {}", avl.key[pivots[0]]));
        }
    }
    rec.hide_marker(t, "inserted");
    rec.fill(t, "default");
    rec.frame("done");
    rec
}

#[allow(dead_code)]
fn main() {
    record().save("avl.json").unwrap();
    println!("wrote avl.json");
}

//! Implementation of a persistent stack, inspired by
//! [Learning Rust With Entirely Too Many Linked
//! Lists](http://cglab.ca/~abeinges/blah/too-many-lists/book/third.html). The stack contains
//! reference counted nodes, implements push/append and pop/tail but never mutates itself.
//! The implementation has been modified to contain a field `len` to enable getting the size of the
//! stack in `O(1)`

use std::fmt;
use std::marker::PhantomData;
use std::sync::Arc;

use serde::ser::{Serialize, SerializeSeq, Serializer};
use serde::de::{Deserialize, Deserializer, SeqAccess, Visitor};

type Link<T> = Option<Arc<Node<T>>>;

/// Persistent stack.
#[derive(Debug, Clone)]
pub struct Stack<T> {
    head: Link<T>,
    len: usize,
}

#[derive(Debug)]
struct Node<T> {
    element: T,
    next: Link<T>,
}

#[derive(Clone)]
pub struct Iter<'a, T>
where
    T: 'a,
{
    next: Option<&'a Node<T>>,
}

impl<T> Stack<T> {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn append(&self, elem: T) -> Self {
        Self {
            head: Some(Arc::new(Node {
                element: elem,
                next: self.head.clone(),
            })),
            len: self.len + 1,
        }
    }

    pub fn iter(&self) -> Iter<T> {
        Iter {
            next: self.head.as_ref().map(|node| &**node),
        }
    }

    pub fn tail(&self) -> (Option<&T>, Self) {
        let new_len = if self.head.is_some() {
            self.len - 1
        } else {
            self.len
        };
        (
            self.head(),
            Self {
                head: self.head.as_ref().and_then(|node| node.next.clone()),
                len: new_len,
            },
        )
    }

    pub fn head(&self) -> Option<&T> {
        self.head.as_ref().map(|node| &node.element)
    }

    pub fn len(&self) -> usize {
        self.len
    }
}

impl<'a, T> Iterator for Iter<'a, T> {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        self.next.map(|node| {
            self.next = node.next.as_ref().map(|node| &**node);
            &node.element
        })
    }
}

impl<T> Drop for Stack<T> {
    fn drop(&mut self) {
        let mut head = self.head.take();
        while let Some(node) = head {
            if let Ok(mut node) = Arc::try_unwrap(node) {
                head = node.next.take();
            } else {
                break;
            }
        }
    }
}

impl<T> Default for Stack<T> {
    fn default() -> Self {
        Self { head: None, len: 0 }
    }
}

impl<T> ::serde::Serialize for Stack<T>
where
    T: Serialize,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut seq = serializer.serialize_seq(Some(self.len()))?;
        for e in self.iter() {
            seq.serialize_element(e)?;
        }
        seq.end()
    }
}

struct StackVisitor<T>
where
    T: Serialize,
{
    marker: PhantomData<fn() -> Stack<T>>,
}

impl<T> StackVisitor<T>
where
    T: Serialize,
{
    fn new() -> Self {
        Self {
            marker: PhantomData,
        }
    }
}

impl<'de, T> Visitor<'de> for StackVisitor<T>
where
    T: Deserialize<'de> + Serialize,
{
    // The type that our Visitor is going to produce.
    type Value = Stack<T>;

    // Format a message stating what data this Visitor expects to receive.
    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("a persistent stack")
    }

    // Deserialize MyMap from an abstract "map" provided by the
    // Deserializer. The MapAccess input is a callback provided by
    // the Deserializer to let us see each entry in the map.
    fn visit_seq<S>(self, mut access: S) -> Result<Self::Value, S::Error>
    where
        S: SeqAccess<'de>,
    {
        // let mut map = MyMap::with_capacity(access.size_hint().unwrap_or(0));
        let mut vec = Vec::with_capacity(access.size_hint().unwrap_or(0));
        let mut chain = Stack::new();

        // While there are entries remaining in the input, add them
        // into our map.
        while let Some(value) = access.next_element()? {
            vec.push(value);
            // chain = chain.append(value);
        }

        for val in vec.into_iter().rev() {
            chain = chain.append(val);
        }

        Ok(chain)
    }
}

// This is the trait that informs Serde how to deserialize MyMap.
impl<'de, T> Deserialize<'de> for Stack<T>
where
    T: Serialize + Deserialize<'de>,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        // Instantiate our Visitor and ask the Deserializer to drive
        // it over the input data, resulting in an instance of MyMap.
        deserializer.deserialize_seq(StackVisitor::new())
    }
}

impl<T> PartialEq for Stack<T>
where
    T: PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        self.len() == other.len() && self.iter().zip(other.iter()).all(|(lhs, rhs)| lhs == rhs)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use quickcheck::{Arbitrary, Gen};

    #[test]
    fn new() {
        let chain: Stack<bool> = Stack::new();
        assert_eq!(chain.len(), 0);
    }

    #[test]
    fn append() {
        let chain = Stack::new();
        assert_eq!(chain.head(), None);
        let chain = chain.append(false);
        assert_eq!(chain.head(), Some(&false));
    }

    #[test]
    fn tail() {
        let chain = Stack::new().append(1).append(2).append(3);
        let (head, chain) = chain.tail();
        assert_eq!(head, Some(&3));
        let (head, chain) = chain.tail();
        assert_eq!(head, Some(&2));
        let (head, chain) = chain.tail();
        assert_eq!(head, Some(&1));
        let (head, chain) = chain.tail();
        assert_eq!(head, None);
        let chain = chain.append(1);
        assert_eq!(chain.head(), Some(&1));
        let (_head, chain) = chain.tail();
        assert_eq!(chain.head(), None);
    }

    #[test]
    fn len() {
        let chain = Stack::new();
        assert_eq!(chain.len(), 0);
        let chain = chain.tail().1;
        assert_eq!(chain.len(), 0);
        let chain = chain.append(3).append(2).append(1);
        assert_eq!(chain.len(), 3);
        let chain = chain.tail().1;
        assert_eq!(chain.len(), 2);
        let chain = chain.tail().1;
        assert_eq!(chain.len(), 1);
        let chain = chain.tail().1;
        assert_eq!(chain.len(), 0);
        let chain = chain.tail().1;
        assert_eq!(chain.len(), 0);
        let chain = chain.append(3).append(2).append(1);
        assert_eq!(chain.len(), 3);
    }

    #[test]
    fn iter() {
        let chain = Stack::new();
        let mut iter = chain.iter();
        assert_eq!(iter.next(), None);
        assert_eq!(iter.next(), None);
        let chain = chain.append(1).append(2).append(3);
        let mut iter = chain.iter();
        assert_eq!(iter.next(), Some(&3));
        assert_eq!(iter.next(), Some(&2));
        assert_eq!(iter.next(), Some(&1));
        assert_eq!(iter.next(), None);
        assert_eq!(iter.next(), None);
    }

    impl<A> Arbitrary for Stack<A>
    where
        A: Arbitrary + ::std::marker::Sync + Default + ::serde::Serialize,
    {
        fn arbitrary<G: Gen>(g: &mut G) -> Self {
            let size = {
                let s = g.size();
                g.gen_range(0, s)
            };
            (0..size).fold(Stack::new(), |acc, _| acc.append(Arbitrary::arbitrary(g)))
        }
    }

    quickcheck! {
        fn append_and_tail_is_identity(xs: Stack<bool>) -> bool {
            let c2 = xs.append(false).tail().1;
            c2 == xs
        }
    }

    quickcheck! {
        fn tail_len(xs: Stack<bool>) -> bool {
            let len = xs.len();
            len >= xs.tail().1.len()
        }
    }

    quickcheck! {
        fn append_increases_length(xs: Stack<bool>) -> bool {
            let len = xs.len();
            len + 1 == xs.append(false).len()
        }
    }
}

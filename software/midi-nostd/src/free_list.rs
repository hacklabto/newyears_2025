#[allow(unused)]
pub trait FreeList {
    fn alloc(self: &mut Self) -> usize;
    fn free(self: &mut Self, item_to_free: usize);
}

#[allow(unused)]
pub struct FreeListImpl<const N: usize> {
    free_list: [Option<usize>; N],
    free_list_head: Option<usize>,
}

#[allow(unused)]
impl<const N: usize> FreeList for FreeListImpl<N> {
    fn alloc(self: &mut Self) -> usize {
        let allocated_item = self
            .free_list_head
            .expect("Unhandled out of sound pool error");
        self.free_list_head = self.free_list[allocated_item];
        self.free_list[allocated_item] = None;
        allocated_item
    }
    fn free(self: &mut Self, item_to_free: usize) {
        assert!(self.free_list[item_to_free].is_none());
        self.free_list[item_to_free] = self.free_list_head;
        self.free_list_head = Some(item_to_free);
    }
}

#[allow(unused)]
impl<const N: usize> FreeListImpl<N> {
    pub fn new() -> Self {
        let free_list: [Option<usize>; N] =
            core::array::from_fn(|idx| if idx == N - 1 { None } else { Some(idx + 1) });
        let free_list_head: Option<usize> = Some(0);

        Self {
            free_list,
            free_list_head,
        }
    }
}

#[cfg(test)]
mod free_list_tests {
    use crate::free_list::*;
    #[test]
    fn free_list_should_alloc_and_free() {
        let mut free_list: FreeListImpl<3> = FreeListImpl::new();
        assert_eq!(0, free_list.alloc());
        assert_eq!(1, free_list.alloc());
        assert_eq!(2, free_list.alloc());
        free_list.free(1);
        free_list.free(0);
        free_list.free(2);
        assert_eq!(2, free_list.alloc());
        assert_eq!(0, free_list.alloc());
        assert_eq!(1, free_list.alloc());
    }
}


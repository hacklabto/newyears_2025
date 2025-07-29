pub struct FreeList<const N: usize> {
    active_list: [bool; N],
    free_list: [Option<usize>; N],
    free_list_head: Option<usize>,
}

impl<const N: usize> FreeList<N> {
    pub fn alloc(self: &mut Self) -> usize {
        let allocated_item = self
            .free_list_head
            .expect("Unhandled out of sound pool error");
        self.free_list_head = self.free_list[allocated_item];
        self.free_list[allocated_item] = None;
        self.active_list[allocated_item] = true;
        allocated_item
    }
    pub fn free(self: &mut Self, item_to_free: usize) {
        assert!(self.free_list[item_to_free].is_none());
        self.free_list[item_to_free] = self.free_list_head;
        self.free_list_head = Some(item_to_free);
        self.active_list[item_to_free] = false;
    }
    pub fn is_active(self: &Self, idx: usize) -> bool {
        self.active_list[idx]
    }
}

impl<const N: usize> Default for FreeList<N> {
    fn default() -> Self {
        let active_list: [bool; N] = core::array::from_fn(|_idx| false);
        let free_list: [Option<usize>; N] =
            core::array::from_fn(|idx| if idx == N - 1 { None } else { Some(idx + 1) });
        let free_list_head: Option<usize> = Some(0);

        Self {
            active_list,
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
        let mut free_list: FreeList<3> = FreeList::default();
        for idx in 0..3 {
            assert_eq!(false, free_list.is_active(idx));
        }
        assert_eq!(0, free_list.alloc());
        assert_eq!(true, free_list.is_active(0));
        assert_eq!(1, free_list.alloc());
        assert_eq!(true, free_list.is_active(1));
        assert_eq!(2, free_list.alloc());
        assert_eq!(true, free_list.is_active(2));
        free_list.free(1);
        assert_eq!(false, free_list.is_active(1));
        free_list.free(0);
        assert_eq!(false, free_list.is_active(0));
        free_list.free(2);
        for idx in 0..3 {
            assert_eq!(false, free_list.is_active(idx));
        }
        assert_eq!(2, free_list.alloc());
        assert_eq!(0, free_list.alloc());
        assert_eq!(1, free_list.alloc());
    }
}

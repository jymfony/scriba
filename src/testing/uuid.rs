use std::cell::RefCell;
use uuid::Uuid;

thread_local!(
    static UUID_LOW_BITS: RefCell<u64> = RefCell::new(0);
);

pub fn generate_test_uuid() -> Uuid {
    let low_bits = UUID_LOW_BITS.with(|lb| {
        let val = *lb.borrow();
        UUID_LOW_BITS.set(val + 1);

        val
    });

    Uuid::from_u64_pair(0, low_bits)
}

pub fn reset_test_uuid() {
    UUID_LOW_BITS.set(0);
}

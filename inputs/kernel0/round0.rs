// preprocessing
const EBUSY: i32 = 16;

// struct/typedef definitions.
type size_t = i32;

type __kernel_loff_t = i64;
type loff_t = __kernel_loff_t;

struct ctl_table<'a> {
    data: &'a mut i32,
    maxlen: i32,
    extra1: &'a mut i32,
    extra2: &'a mut i32,
}

struct atomic_t {
    counter: i32,
}
type atomic_long_t = atomic_t;

struct arch_spinlock_t {
    lock: u32,
}

struct raw_spinlock_t {
    raw_lock: arch_spinlock_t,
}

struct list_head<'a> {
    next: &'a mut list_head<'a>,
    prev: &'a mut list_head<'a>,
}

struct mutex<'a> {
    owner: atomic_long_t,
    wait_lock: raw_spinlock_t,
    wait_list: list_head<'a>,
}

// function prototypes
fn proc_dointvec_minmax(
    table: &mut ctl_table,
    write: i32,
    buffer: &mut i32,
    len: &mut size_t,
    pos: &mut loff_t,
) -> i32 {
    return 0;
}
fn mutex_lock(lock: &mut mutex) {}
fn atomic_read(v: &mut atomic_t) -> i32 {
    return 0;
}
fn mutex_unlock(lock: &mut mutex) {}

// global variables
// Moved to function parameters to avoid Rust Static variable checks.

// function to check, from callchain.c
fn perf_event_max_stack_handler(
    table: &mut ctl_table,
    write: i32,
    buffer: &mut i32,
    lenp: &mut size_t,
    ppos: &mut loff_t,
    mut nr_callchain_events: atomic_t,
    mut callchain_mutex: mutex,
) -> i32 {
    let value: &mut i32 = (*table).data;
    let mut new_value: i32 = *value;
    let mut ret: i32;
    let mut new_table: ctl_table = *table; // Cannout move out of *table behind mutable reference

    new_table.data = &mut new_value;
    ret = proc_dointvec_minmax(&mut new_table, write, buffer, lenp, ppos);
    if ret != 0 || write == 0 {
        return ret;
    }

    mutex_lock(&mut callchain_mutex);
    if atomic_read(&mut nr_callchain_events) != 0 {
        ret = -EBUSY;
    } else {
        *value = new_value;
    }
    mutex_unlock(&mut callchain_mutex);

    return ret;
}

fn main() {
    println!("Runs!");
}

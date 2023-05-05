type size_t = u64;
type sector_t = u64;

struct seqlock_t {
    filler: i32,
}

struct badblocks<'a> {
    count: i32,
    unacked_exist: i32,
    shift: i32,
    page: &'a mut u64,
    changed: i32,
    lock: seqlock_t,
}

const BB_LEN_MASK: u64 = (0x00000000000001FF);
const BB_OFFSET_MASK: u64 = (0x7FFFFFFFFFFFFE00);
const BB_ACK_MASK: u64 = (0x8000000000000000);
const BB_MAX_LEN: u64 = 512;

fn BB_OFFSET(x: u64) -> u64 {
    ((x) & BB_OFFSET_MASK) >> 9
}

fn BB_LEN(x: u64) -> u64 {
    (((x) & BB_LEN_MASK) + 1)
}

fn BB_ACK(x: u64) -> u64 {
    (!!((x) & BB_ACK_MASK))
}

fn BB_MAKE(a: u64, l: u64, ack: u64) -> u64 {
    (((a) << 9) | ((l) - 1) | ((!!(ack)) << 63))
}

const PAGE_SIZE: u64 = 10;
const MAX_BADBLOCKS: u64 = (PAGE_SIZE / 8);

fn write_seqlock_irqsave(lock: &mut seqlock_t, flags: i32) {}

fn __write_seqlock_irqsave(sl: &mut seqlock_t) {}
fn memmove(a: &mut i32, b: &mut i32, c: size_t) {}
fn badblocks_update_acked(bb: &badblocks) {}
fn write_sequnlock_irqrestore(sl: &mut seqlock_t, flags: u64) {}

// from badblocks.c
fn badblocks_set(bb: &mut badblocks, mut s: sector_t, mut sectors: u64, acknowledged: i32) -> i32 {
    let mut p: &mut u64;
    let mut lo: i32;
    let mut hi: i32;
    let mut rv = 0;
    let mut flags: i32 = 0;

    if (*bb).shift < 0 {
        return 1;
    }

    if (*bb).shift != 0 {
        let mut next: sector_t = s + sectors;

        s >>= (*bb).shift;
        next += (1 << (*bb).shift) - 1;
        next >>= (*bb).shift;
        sectors = next - s;
    }

    write_seqlock_irqsave(&mut (*bb).lock, flags);

    p = (*bb).page;
    lo = 0;
    hi = (*bb).count;
    while hi - lo > 1 {
        let mid = (lo + hi) / 2;
        let a: sector_t = BB_OFFSET(*p);

        if a <= s {
            lo = mid;
        } else {
            hi = mid;
        }
    }
    if (hi > lo && BB_OFFSET(*p) > s) {
        hi = lo;
    }

    if (hi > lo) {
        let mut a: sector_t = BB_OFFSET(*p);
        let mut e: sector_t = a + BB_LEN(*p);
        let mut ack: u64 = BB_ACK(*p);

        if (e >= s) {
            if (s == a && s + sectors >= e) {
                ack = acknowledged as u64;
            } else {
                ack = (ack != 0 && acknowledged != 0) as u64;
            }

            if (e < s + sectors) {
                e = s + sectors;
            }
            if (e - a <= BB_MAX_LEN) {
                *p = BB_MAKE(a, e - a, ack);
                s = e;
            } else {
                if (BB_LEN(*p) != BB_MAX_LEN) {
                    *p = BB_MAKE(a, BB_MAX_LEN, ack);
                }
                s = a + BB_MAX_LEN;
            }
            sectors = e - s;
        }
    }

    (*bb).changed = 1;
    if acknowledged == 0 {
        (*bb).unacked_exist = 1;
    } else {
        badblocks_update_acked(bb);
    }
    write_sequnlock_irqrestore(&mut (*bb).lock, flags as u64);

    return rv;
}

fn main() {
    println!("Runs!");
}

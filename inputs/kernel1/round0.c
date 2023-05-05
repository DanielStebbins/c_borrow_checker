typedef long long size_t;
typedef unsigned long long	u64;
typedef u64 sector_t;

typedef struct {
	int filler;
} seqlock_t;

struct badblocks {
	int count;
	int unacked_exist;
	int shift;
	u64 *page;
	int changed;
	seqlock_t lock;
};

#define BB_LEN_MASK	(0x00000000000001FFULL)
#define BB_OFFSET_MASK	(0x7FFFFFFFFFFFFE00ULL)
#define BB_ACK_MASK	(0x8000000000000000ULL)
#define BB_MAX_LEN	512
#define BB_OFFSET(x)	(((x) & BB_OFFSET_MASK) >> 9)
#define BB_LEN(x)	(((x) & BB_LEN_MASK) + 1)
#define BB_ACK(x)	(!!((x) & BB_ACK_MASK))
#define BB_MAKE(a, l, ack) (((a)<<9) | ((l)-1) | ((u64)(!!(ack)) << 63))

#define PAGE_SIZE 10
#define MAX_BADBLOCKS	(PAGE_SIZE/8)

#define write_seqlock_irqsave(lock, flags)				\
	do { flags = __write_seqlock_irqsave(lock); } while (0)

unsigned long __write_seqlock_irqsave(seqlock_t *sl);
void *memmove(void *a, const void *b, size_t c);
void badblocks_update_acked(struct badblocks *bb);
void write_sequnlock_irqrestore(seqlock_t *sl, unsigned long flags);







// from badblocks.c
int badblocks_set(struct badblocks *bb, sector_t s, int sectors, int acknowledged)
{
	u64 *p;
	int lo, hi;
	int rv = 0;
	unsigned long flags;

	if ((*bb).shift < 0) {
		return 1;
	}

	if ((*bb).shift) {
		sector_t next = s + sectors;

		s >>= (*bb).shift;
		next += (1<<(*bb).shift) - 1;
		next >>= (*bb).shift;
		sectors = next - s;
	}

	write_seqlock_irqsave(&(*bb).lock, flags);

	p = (*bb).page;
	lo = 0;
	hi = (*bb).count;
	while (hi - lo > 1) {
		int mid = (lo + hi) / 2;
		sector_t a = BB_OFFSET(p[mid]);

		if (a <= s) {
			lo = mid;
		}
		else {
			hi = mid;
		}
	}
	if (hi > lo && BB_OFFSET(p[lo]) > s) {
		hi = lo;
	}

	if (hi > lo) {
		sector_t a = BB_OFFSET(p[lo]);
		sector_t e = a + BB_LEN(p[lo]);
		int ack = BB_ACK(p[lo]);

		if (e >= s) {
			if (s == a && s + sectors >= e) {
				ack = acknowledged;
			}
			else {
				ack = ack && acknowledged;
			}

			if (e < s + sectors) {
				e = s + sectors;
			}
			if (e - a <= BB_MAX_LEN) {
				p[lo] = BB_MAKE(a, e-a, ack);
				s = e;
			} else {
				if (BB_LEN(p[lo]) != BB_MAX_LEN) {
					p[lo] = BB_MAKE(a, BB_MAX_LEN, ack);
				}
				s = a + BB_MAX_LEN;
			}
			sectors = e - s;
		}
	}
	if (sectors && hi < (*bb).count) {
		sector_t a = BB_OFFSET(p[hi]);
		sector_t e = a + BB_LEN(p[hi]);
		int ack = BB_ACK(p[hi]);

		if (a <= s + sectors) {
			if (e <= s + sectors) {
				e = s + sectors;
				ack = acknowledged;
			} else {
				ack = ack && acknowledged;
			}

			a = s;
			if (e - a <= BB_MAX_LEN) {
				p[hi] = BB_MAKE(a, e-a, ack);
				s = e;
			} else {
				p[hi] = BB_MAKE(a, BB_MAX_LEN, ack);
				s = a + BB_MAX_LEN;
			}
			sectors = e - s;
			lo = hi;
			hi++;
		}
	}
	if (sectors == 0 && hi < (*bb).count) {
		sector_t a = BB_OFFSET(p[hi]);
		int lolen = BB_LEN(p[lo]);
		int hilen = BB_LEN(p[hi]);
		int newlen = lolen + hilen - (s - a);

		if (s >= a && newlen < BB_MAX_LEN) {
			/* yes, we can combine them */
			int ack = BB_ACK(p[lo]) && BB_ACK(p[hi]);

			p[lo] = BB_MAKE(BB_OFFSET(p[lo]), newlen, ack);
			memmove(p + hi, p + hi + 1,
				((*bb).count - hi - 1) * 8);
			(*bb).count--;
		}
	}
	while (sectors) {
		if ((*bb).count >= MAX_BADBLOCKS) {
			rv = 1;
			break;
		} else {
			int this_sectors = sectors;

			memmove(p + hi + 1, p + hi,
				((*bb).count - hi) * 8);
			(*bb).count++;

			if (this_sectors > BB_MAX_LEN) {
				this_sectors = BB_MAX_LEN;
			}
			p[hi] = BB_MAKE(s, this_sectors, acknowledged);
			sectors -= this_sectors;
			s += this_sectors;
		}
	}

	(*bb).changed = 1;
	if (!acknowledged) {
		(*bb).unacked_exist = 1;
	}
	else {
		badblocks_update_acked(bb);
	}
	write_sequnlock_irqrestore(&(*bb).lock, flags);

	return rv;
}
// preprocessing
#define	EBUSY		16



// struct/typedef definitions.
typedef int size_t;

typedef long long	__kernel_loff_t;
typedef __kernel_loff_t		loff_t;

struct ctl_table {
	void *data;
	int maxlen;
	void *extra1;
	void *extra2;
};

typedef struct {
	int counter;
} atomic_t;
typedef atomic_t atomic_long_t;

typedef struct {
	unsigned int lock;
} arch_spinlock_t;

typedef struct raw_spinlock {
	arch_spinlock_t raw_lock;
} raw_spinlock_t;

struct list_head {
	struct list_head *next, *prev;
};

struct mutex {
	atomic_long_t		owner;
	raw_spinlock_t		wait_lock;
	struct list_head	wait_list;
};





// function prototypes
int proc_dointvec_minmax(struct ctl_table *table, int write, void *buffer, size_t *len, loff_t *pos);
void mutex_lock(struct mutex *lock);
int atomic_read(const atomic_t *v);
void mutex_unlock(struct mutex *lock);


// global variables
atomic_t nr_callchain_events;
struct mutex callchain_mutex;


// function to check, from callchain.c
int perf_event_max_stack_handler(struct ctl_table *table, int write, void *buffer, size_t *lenp, loff_t *ppos) { 
	int *value = (*table).data;
	int new_value = *value, ret;
	struct ctl_table new_table = *table;

	new_table.data = &new_value;
	ret = proc_dointvec_minmax(&new_table, write, buffer, lenp, ppos);
	if (ret || !write)
		return ret;

	mutex_lock(&callchain_mutex);
	if (atomic_read(&nr_callchain_events))
		ret = -EBUSY;
	else
		*value = new_value;

	mutex_unlock(&callchain_mutex);

	return ret;
}
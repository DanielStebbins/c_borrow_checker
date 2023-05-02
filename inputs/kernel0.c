// the parser requires all types to be defined.
struct ctl_table {
	int *data;
};

typedef struct loff {

} loff_t;

typedef int size_t;             // the parser does not recognize size_t.

// from callchain.c
int perf_event_max_stack_handler(struct ctl_table *table, int write, void *buffer, size_t *lenp, loff_t *ppos) { 
	int *value = table.data;
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
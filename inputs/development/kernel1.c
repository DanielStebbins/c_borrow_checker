struct module *__module_address(unsigned long addr)
{
	struct module *mod;
	struct mod_tree_root *tree;

	if (addr >= mod_tree.addr_min && addr <= mod_tree.addr_max) {
		tree = &mod_tree;
    }
	else if (addr >= mod_data_tree.addr_min && addr <= mod_data_tree.addr_max) {
		tree = &mod_data_tree;
    }
	else {
		return NULL;
    }

	module_assert_mutex_or_preempt();

	mod = mod_find(addr, tree);
	if (mod) {
		BUG_ON(!within_module(addr, mod));
		if (mod->state == MODULE_STATE_UNFORMED) {
			mod = NULL;
        }
	}
	return mod;
}
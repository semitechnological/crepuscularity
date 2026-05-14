module crepuscularity

import json
import os

pub struct ViewIr {
pub:
	version int
}

fn crepus_bin() string {
	return os.getenv_opt('CREPUS_BIN') or { 'crepus' }
}

pub fn render_ir(path string, context map[string]string) !ViewIr {
	ctx_path := os.join_path(os.temp_dir(), 'crepus-context-${os.getpid()}.json')
	os.write_file(ctx_path, json.encode(context))!
	defer {
		os.rm(ctx_path) or {}
	}
	res := os.execute('${os.quoted_path(crepus_bin())} native ir ${os.quoted_path(path)} --ctx ${os.quoted_path(ctx_path)}')
	if res.exit_code != 0 {
		return error(res.output)
	}
	return json.decode(ViewIr, res.output)!
}

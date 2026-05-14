module crepuscularity

fn test_render_ir() {
	ir := render_ir('plugins/fixtures/hello.crepus', {
		'name': 'Ada'
	})!
	assert ir.version == 2
}

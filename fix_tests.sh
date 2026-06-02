#!/bin/bash
# Revert the double make_mut call that happened accidentally in the tests
sed -i 's/std::sync::Arc::make_mut(&mut std::sync::Arc::make_mut(&mut ctx.virtual_files))/std::sync::Arc::make_mut(\&mut ctx.virtual_files)/g' crates/crepuscularity-core/src/virtual_files.rs

import('./crate/pkg').then(module => {
	let roms = new Map();
	roms.set('IBM', './roms/ibm.c8');
	roms.set('TETRIS', './roms/tetris.c8');

	console.log(roms);

	let chip8 = new module.Chip8();
	chip8.reset();
	fetch(roms.get("TETRIS"))
		.then((res) => {
			return res.arrayBuffer();
		}).then((arrayBuffer) => {
			let data = new Uint8Array(arrayBuffer);
			chip8.load(data);
			chip8.set_speed(4);
			
			let keyMap = new Map();
			keyMap.set(88, 0);
			keyMap.set(49, 1);
			keyMap.set(50, 2);
			keyMap.set(51, 3);
			keyMap.set(81, 4);
			keyMap.set(87, 5);
			keyMap.set(69, 6);
			keyMap.set(65, 7);
			keyMap.set(82, 13);
			
			
			window.addEventListener('keydown', function(e){
				let value = keyMap.get(e.keyCode);
				console.log(e);
				if(value){
					chip8.set_key(value, true);
				}
			});
			
			window.addEventListener('keyup', function(e){
				let value = keyMap.get(e.keyCode);
				if(value){
					chip8.set_key(value, false);
				}
			});
			
			setInterval(function(){
				let data = chip8.get_gfx_data();
				let ctx = document.getElementById('canvas').getContext('2d');
				ctx.fillStyle = "black";
				ctx.fillRect(0, 0, ctx.canvas.width, ctx.canvas.height);
				for(var i = 0; i != data.length; i++){
					ctx.fillStyle = "red";
					if(data[i]){
						ctx.fillRect((i % 64) * 10, ((i / 64) | 0) * 10, 10, 10);
					}
				}
				chip8.cycle();
			}, 1000/60);
		});


	console.log(chip8);
	console.log(module);
});
import module from './crate/Cargo.toml';
let roms = new Map();
roms.set("IBM", '.' + require('../roms/ibm.c8'));

console.log(roms);

let chip8 = new module.Chip8();
chip8.reset();
fetch(roms.get("IBM"))
	.then((res) => {
		return res.arrayBuffer();
	}).then((arrayBuffer) => {
		let data = new Uint8Array(arrayBuffer);
		chip8.load(data);
		setInterval(function(){
			chip8.cycle();
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
			
		}, 1000/60);
	});


console.log(chip8);
console.log(module);
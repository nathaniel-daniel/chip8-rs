export class CHIP8{
	static get fontset(){
		return [
			0xF0, 0x90, 0x90, 0x90, 0xF0, //0
			0x20, 0x60, 0x20, 0x20, 0x70, //1
			0xF0, 0x10, 0xF0, 0x80, 0xF0, //2
			0xF0, 0x10, 0xF0, 0x10, 0xF0, //3
			0x90, 0x90, 0xF0, 0x10, 0x10, //4
			0xF0, 0x80, 0xF0, 0x10, 0xF0, //5
			0xF0, 0x80, 0xF0, 0x90, 0xF0, //6
			0xF0, 0x10, 0x20, 0x40, 0x40, //7
			0xF0, 0x90, 0xF0, 0x90, 0xF0, //8
			0xF0, 0x90, 0xF0, 0x10, 0xF0, //9
			0xF0, 0x90, 0xF0, 0x90, 0x90, //A
			0xE0, 0x90, 0xE0, 0x90, 0xE0, //B
			0xF0, 0x80, 0x80, 0x80, 0xF0, //C
			0xE0, 0x90, 0x90, 0x90, 0xE0, //D
			0xF0, 0x80, 0xF0, 0x80, 0xF0, //E
			0xF0, 0x80, 0xF0, 0x80, 0x80  //F
		];
	}
	constructor(){
		this.gfx = new Uint8Array(64 * 32);
		this.stack = new Uint16Array(16);
		this.key = new Uint8Array(16);
		this.V = new Uint8Array(16);
		this.memory = new Uint8Array(4096);
	}
	init(){
		this.pc = 0x200;
		this.opcode = 0;
		this.I = 0;
		this.sp = 0;
		this.delayTimer = 0;
		this.soundTimer = 0;
		this.drawFlag = true;
		for(let i = 0; i != 64 * 32; i++){
			this.gfx[i] = 0;
		}
		for(let i = 0; i != 16; i++){
			this.stack[i] = 0;
		}
		for(let i = 0; i != 16; i++){
			this.key[i] = 0;
		}
		for(let i = 0; i != 16; i++){
			this.V[i] = 0;
		}
		for(let i = 0; i != this.constructor.fontset.length; i++){
			this.memory[i] = this.constructor.fontset[i];
		}
	}
	load(path, cb){
		this.init();
		let self = this;
		let req = new XMLHttpRequest();
		req.open("GET", path);
		req.responseType = 'arraybuffer';
		req.onload = function(){
			//TODO: Check size
			let rom = new Uint8Array(this.response);
			for(let i = 0; i != rom.length; i++){
				self.memory[i + 512] = rom[i];
			}
			if(cb){
				cb();
			}
		};
		req.send();
	}
				cycle(){
					this.pc &= 0x0FFF;
					//-60- ~500 instructions/sec, timers operate at 60, emulate -3- 10 cycles per clock tick
					this.opcode = this.memory[this.pc] << 8 | this.memory[this.pc + 1];
					switch(this.opcode & 0xF000){
						case 0x0000:
							switch(this.opcode & 0x00F){
								case 0x0000:
									for(let i = 0; i != 64 * 32; i++){
										this.gfx[i] = 0x0;
									}
									this.drawFlag = true;
									this.pc += 2;
									break;
								case 0x000E:
									this.sp--;
									this.pc = this.stack[this.sp];
									this.pc += 2;
									break;
								default:
									this.unknownOpcode(this.opcode);
							}
							break;
						case 0x1000:
							this.pc = this.opcode & 0x0FFF;
							break;
						case 0x2000: 
							this.stack[this.sp] = this.pc + 2;
							this.sp++;
							this.pc = (this.opcode & 0x0FFF);
							break;
						case 0x3000: //Check implementation
							if(((this.opcode & 0x0F00) >> 8) == (this.opcode & 0x00FF)){
								this.pc += 2;
							}
							this.pc += 2;
							break;
						case 0x4000:
							if(((this.opcode & 0x0F00) >> 8) != (this.opcode & 0x00FF)){
								this.pc += 2;
							}
							this.pc += 2;
							break;
						case 0x6000:
							this.V[(this.opcode & 0x0F00) >> 8] = this.opcode & 0x00FF;
							this.pc += 2;
							break;
						case 0x7000:
							this.V[(this.opcode & 0x0F00) >> 8] += (this.opcode & 0x00FF);
							this.V[(this.opcode & 0x0F00) >> 8] &= 0xFF; //limit to 1 byte
							this.pc += 2;
							break;
						case 0xA000: // ANNN: Sets I to the address NNN
							this.I = this.opcode & 0x0FFF;
							this.pc += 2;
							break;
						case 0xD000:
							let x = this.V[(this.opcode & 0x0F00) >> 8];
							let y = this.V[(this.opcode & 0x00F0) >> 4];
							let height = this.opcode & 0x000F;
							let pixel = 0;
							
							this.V[0xF] = 0;
							
							for(let yLine = 0; yLine != height; yLine++){
								pixel = this.memory[this.I + yLine];
								for(let xLine = 0; xLine != 8; xLine++){
									if((pixel & (0x80 >> xLine)) != 0){
										if(this.gfx[x + xLine + ((y + yLine) * 64)] == 1){
											this.V[0xF] = 1;
										}
										this.gfx[x + xLine + ((y + yLine) * 64)] ^= 1;
									}
								}
							}
							
							this.drawFlag = true;
							this.pc += 2;
							break;
						case 0xF000:
							switch(this.opcode & 0x00FF){
								case 0x001E:
									if(this.V[(this.opcode & 0x0F00) >> 8] + this.I > 0x0FFF){
										this.V[0xF] = 1;
									}else {
										this.V[0xF] = 0;
									}
									this.I += this.V[(this.opcode & 0x0F00) >> 8];
									this.pc += 2;
									break;
								case 0x0029:
									this.I = this.memory[((this.opcode & 0x0F00) >> 8)] * 5;
									this.pc +=2;
									break;
								case 0x0033:
									this.memory[this.I] = (this.V[(this.opcode & 0x0F00) >> 8] / 100) | 0;
									this.memory[this.I + 1] = ((this.V[(this.opcode & 0x0F00) >> 8] / 10) | 0) % 100;
									this.memory[this.I + 2] = ((this.V[(this.opcode & 0x0F00) >> 8]) % 100) % 10;
									this.pc += 2;
									break;
								case 0x0065:
									for(let i = 0; i != ((this.opcode & 0x0F00) >> 8); i++){
										this.V[i] = this.memory[i + this.I];
									}
									this.I += ((this.opcode & 0x0F00) >> 8) + 1;
									this.pc += 2;
									break;
								default:
									this.unknownOpcode(this.opcode);
							}
							break;
						default:
							this.unknownOpcode(this.opcode);
					}
					
					
				}
	updateTimers(){
		if(this.delayTimer > 0){
			this.delayTimer--;
		}
		if(this.soundTimer > 0){
			soundTimer--;
		}
	}
	setKeys(){
					
	}
	unknownOpcode(opcode){
		throw ("Unknown Opcode: 0x" + opcode.toString(16).padStart(4, '0').toUpperCase() + " (" + opcode + ")");
	}
}

//Utils
export function runChip(opts){
	let chip = opts.chip;
	let canvas = opts.canvas;
	let debug = opts.debug || false;
	
	let ctx = canvas.getContext('2d');
	
	function draw(){
		if(chip.drawFlag){
				ctx.clearRect(0, 0, ctx.canvas.width, ctx.canvas.height);
				for(let y = 0; y != 32; y++){
					for(let x = 0; x != 64; x++){
						if((chip.gfx[(y * 64) + x]) == 0){
							ctx.fillStyle = "black";
						}else{
							ctx.fillStyle = "grey";
						}
						ctx.fillRect(x, y, 1, 1, 1, 1);	
					}
				}
			}
	}
	
	if(debug){
		let instructionLimit = debug.instructionLimit || Infinity;
		let history = [];
		return function(){
			let loop = setInterval(function(){
				for(let i = 0; i != instructionLimit; i++){
					try{
						chip.cycle();
					}catch(e){
						console.error(e);
						clearInterval(loop);
					}
					history.push('0x'+ a.opcode.toString(16).toUpperCase().padStart(4, '0'));
				}
				chip.updateTimers();
				draw();
				chip.drawFlag = false;
				if(history.length > instructionLimit){
					clearInterval(loop);
					console.log("Instruction limit reached, did you forget to increment the pc?");
					console.log("Instruction History: ", history);
					console.log("CHIP: ", chip);
				}
			}, 1000/60);
		}
	}
	return function(){
		setInterval(function(){
			for(let i = 0; i != 10; i++){
				chip.cycle();
			}
			chip.updateTimers();
			draw();
			chip.drawFlag = false;
		}, 1000/60);
	};
}
			
export function hasTrue(a){
	for(let i = 0; i != a.length; i++){
		if(a[i] == 1){
			return i;
			}
	}
	return false;
}

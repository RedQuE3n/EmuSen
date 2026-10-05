// A behavioural stand-in for the MiSTer memory block Nuked-MD's vram.v names: 256 words of 256 bits, a registered
// read, writes by byte enable. Written from the instance's port list, for the board bench (Nephrite_Native.md §10).
module vram_ip (input [7:0] address, input [31:0] byteena, input clock, input [255:0] data, input wren, output reg [255:0] q);
	reg [255:0] mem [0:255];
	integer i;
	always @(posedge clock) begin
		if (wren)
			for (i = 0; i < 32; i = i + 1)
				if (byteena[i]) mem[address][8*i +: 8] <= data[8*i +: 8];
		q <= mem[address];
	end
endmodule

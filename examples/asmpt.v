module test(input clk, input ctrl);

reg[4:0] state;
initial begin
  state = 0;
end

always @(posedge clk) begin
  if (state == 5'b01010)
    state <= 0;
  else if (state >= 5'b00110 && ctrl)
    state <= (state | 5'b10000);
  else
    state <= state + 1;
end

assume property (state != 5'b11101);
assert property (state != 5'b11111);

endmodule

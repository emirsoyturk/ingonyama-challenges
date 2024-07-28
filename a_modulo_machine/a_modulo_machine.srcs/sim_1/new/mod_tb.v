`timescale 1ns / 1ps
//////////////////////////////////////////////////////////////////////////////////
// Company: 
// Engineer: 
// 
// Create Date: 22.06.2024 14:16:46
// Design Name: 
// Module Name: mod_tb
// Project Name: 
// Target Devices: 
// Tool Versions: 
// Description: 
// 
// Dependencies: 
// 
// Revision:
// Revision 0.01 - File Created
// Additional Comments:
// 
//////////////////////////////////////////////////////////////////////////////////


module mod_tb;
    reg                 clk;
    reg                 rst;
    reg     [299:0]     X;
    wire    [255:0]     O;
    
    mod uut (
        .clk(clk),
        .rst(rst),
        .X(X),
        .O(O)
    );
    
    initial begin
        clk = 0;
        forever #5 clk = ~clk;
    end
    
    initial begin
        rst = 1;
        X = 0;
        #10 rst = 0;
        #10 X = 300'h0;
        #10 X = 300'h1;
        #100 $finish;
   end 
endmodule

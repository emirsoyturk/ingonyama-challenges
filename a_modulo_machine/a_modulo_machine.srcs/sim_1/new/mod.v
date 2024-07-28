`timescale 1ns / 1ps
//////////////////////////////////////////////////////////////////////////////////
// Company: 
// Engineer: 
// 
// Create Date: 22.06.2024 14:02:10
// Design Name: 
// Module Name: mod
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


module mod(
    input                               clk,
    input                               rst,
    input                   [299:0]     X,
    output      reg         [255:0]     O
    );
    
    parameter [255:0] P = 256'HE7EB417862865B8FF6FA5C28E93008D69368F209AD2757CC370682FE26BDC75D;
    
    always @(posedge clk) begin
        if (rst) begin
            O <= 0;
        end else begin
            O <= X % P;
        end
    end
endmodule

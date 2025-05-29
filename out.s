global main
extern print_string

section .data
var_0 db "Hello, World!", 0
var_len_0 equ 15

section .text
main:
	push rbp
	mov rbp, rsp

	sub rsp, 8
	lea rax, [rel var_0]
	mov QWORD [rsp], rax

  mov rdi, [rbp - 8]
  call print_string

  mov eax, 0
  leave
  ret
  

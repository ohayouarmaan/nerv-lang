section .data
	LC_0 db "Value of s: %p", 0
	LC_len_0 equ 16
section .text
	extern printf
	extern malloc
	global main
main:
	push rbp
	mov rbp, rsp
	sub rsp, 16

	; VARIABLE DECLARATION
	mov rdi, 4
	xor rax, rax
	call malloc
	mov rax, rax
	mov QWORD [rbp-8], rax


	; VARIABLE REASSIGNMENT
	lea rbx, [rbp-8]
	mov rbx, [rbx]
	mov rdx, 4
	mov [rbx], rdx
; Expression Statement
	lea rdi, [rel LC_0]
	mov rsi, QWORD [rbp-8]
	xor rax, rax
	call printf
	mov rax, rax

	; Return Statement
	mov rax, 0
	leave
	ret

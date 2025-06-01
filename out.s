section .data
	LC_0 db "value for int: %d", 0
	LC_len_0 equ 19
section .text
	extern printf
	extern malloc
	global main
main:
	push rbp
	mov rbp, rsp
	sub rsp, 0
	; VARIABLE DECLARATION
	mov rax, 9
	mov DWORD [rbp-4], eax


	; VARIABLE DECLARATION

	lea rax, [rbp-4]
	mov QWORD [rbp-12], rax


	; VARIABLE DECLARATION

	lea rax, [rbp-12]
	mov QWORD [rbp-20], rax

; Expression Statement
	lea rdi, [rel LC_0]
	mov rsi, QWORD [rbp-20]
	mov rsi, [rsi]
	mov rsi, [rsi]
	xor rax, rax
	call printf
	mov rax, rax

	; Return Statement
	mov rax, 0
	leave
	ret

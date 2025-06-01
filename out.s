section .data
	LC_0 db "Pointer %d", 0
	LC_len_0 equ 12
section .text
	extern printf
	extern malloc
	global main
	global printPointer
printPointer:
	push rbp
	mov rbp, rsp
	sub rsp, 32
	mov QWORD [rbp-8], rdi
	mov DWORD [rbp-12], esi
; Expression Statement
	lea rdi, [rel LC_0]
	mov rsi, QWORD [rbp-8]
	mov rsi, [rsi]
	xor rax, rax
	call printf
	mov rax, rax
	mov rax, 0
	leave
	ret
main:
	push rbp
	mov rbp, rsp
	sub rsp, 32

	; VARIABLE DECLARATION
	mov rax, 9
	mov DWORD [rbp-4], eax


	; VARIABLE DECLARATION

	lea rax, [rbp-4]
	mov QWORD [rbp-12], rax

; Expression Statement
	mov rdi, QWORD [rbp-12]
	mov rsi, 2
	xor rax, rax
	call printPointer
	mov rax, rax

	; Return Statement
	mov rax, 0
	leave
	ret

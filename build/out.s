section .data
	LC_0 db "Value of valid pointer: %d", 0
	LC_len_0 equ 28
	LC_1 db "Value of dangling pointer: %d", 0
	LC_len_1 equ 31
section .text
	extern printf
	extern malloc
	global main
	global returnsPointerToAStackVariable
	global returnsPointerToAHeapAllocatedInt
returnsPointerToAHeapAllocatedInt:
	push rbp
	mov rbp, rsp
	sub rsp, 32
	mov DWORD [rbp-4], edi

	; VARIABLE DECLARATION
	mov rdi, 4
	xor rax, rax
	call malloc
	mov rax, rax
	mov QWORD [rbp-12], rax


	; VARIABLE REASSIGNMENT
	lea rbx, [rbp-12]
	mov rbx, [rbx]
	mov rdx, 45
	mov [rbx], rdx

	; Return Statement
	mov rax, QWORD [rbp-12]
	leave
	ret
main:
	push rbp
	mov rbp, rsp
	sub rsp, 32

	; VARIABLE DECLARATION
	xor rax, rax
	call returnsPointerToAHeapAllocatedInt
	mov rax, rax
	mov QWORD [rbp-8], rax


	; VARIABLE DECLARATION
	xor rax, rax
	call returnsPointerToAStackVariable
	mov rax, rax
	mov QWORD [rbp-16], rax

; Expression Statement
	lea rdi, [rel LC_0]
	mov rsi, QWORD [rbp-8]
	mov rsi, [rsi]
	xor rax, rax
	call printf
	mov rax, rax
; Expression Statement
	lea rdi, [rel LC_1]
	mov rsi, QWORD [rbp-16]
	mov rsi, [rsi]
	xor rax, rax
	call printf
	mov rax, rax

	; Return Statement
	mov rax, 0
	leave
	ret
returnsPointerToAStackVariable:
	push rbp
	mov rbp, rsp
	sub rsp, 16

	; VARIABLE DECLARATION
	mov rax, 4
	mov DWORD [rbp-4], eax


	; Return Statement

	lea rax, [rbp-4]
	leave
	ret

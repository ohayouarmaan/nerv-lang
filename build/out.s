section .data
	LC_0 db "Value of valid pointer: %d", 0
	LC_len_0 equ 28
	LC_1 db "Value of dangling pointer: %d", 0
	LC_len_1 equ 31
section .text
	extern _printf
	extern _malloc
	global _main
	global _returnsPointerToAStackVariable
	global _returnsPointerToAHeapAllocatedInt
_returnsPointerToAHeapAllocatedInt:
	push rbp
	mov rbp, rsp
	sub rsp, 32
	mov DWORD [rbp-4], edi

	; VARIABLE DECLARATION
	mov rdi, 4
	xor rax, rax
	call _malloc
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
_main:
	push rbp
	mov rbp, rsp
	sub rsp, 32

	; VARIABLE DECLARATION
	xor rax, rax
	call _returnsPointerToAHeapAllocatedInt
	mov rax, rax
	mov QWORD [rbp-8], rax


	; VARIABLE DECLARATION
	xor rax, rax
	call _returnsPointerToAStackVariable
	mov rax, rax
	mov QWORD [rbp-16], rax

; Expression Statement
	lea rdi, [rel LC_0]
	mov rsi, QWORD [rbp-8]
	mov rsi, [rsi]
	xor rax, rax
	call _printf
	mov rax, rax
; Expression Statement
	lea rdi, [rel LC_1]
	mov rsi, QWORD [rbp-16]
	mov rsi, [rsi]
	xor rax, rax
	call _printf
	mov rax, rax

	; Return Statement
	mov rax, 0
	leave
	ret
_returnsPointerToAStackVariable:
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

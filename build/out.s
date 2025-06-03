section .data
	LC_0 db "Value of s: %d", 0
	LC_len_0 equ 16
section .text
	extern _printf
	extern _malloc
	global _main
_main:
	push rbp
	mov rbp, rsp
	sub rsp, 32

	; VARIABLE DECLARATION
	mov rdi, 4
	xor rax, rax
	call _malloc
	mov rax, rax
	mov QWORD [rbp-8], rax


	; VARIABLE REASSIGNMENT
	lea rbx, [rbp-8]
	mov rbx, [rbx]
	mov rdx, 4
	mov [rbx], rdx

	; VARIABLE DECLARATION
	mov rax, QWORD [rbp-8]
	mov rax, [rax]
	mov DWORD [rbp-12], eax


	; VARIABLE DECLARATION
	mov rax, QWORD [rbp-8]
	mov rax, [rax]
	mov DWORD [rbp-16], eax


	; VARIABLE DECLARATION
	mov rax, QWORD [rbp-8]
	mov rax, [rax]
	mov DWORD [rbp-20], eax


	; VARIABLE REASSIGNMENT
	lea rbx, [rbp-12]
	mov edx, DWORD [rbp-12]
	push rdx
	mov rdx, 2
	push rdx
	mov rdx, 8
	pop rcx
	imul rdx, rcx
	pop rcx
	add rdx, rcx
	push rdx
	mov rdx, 2
	push rdx
	mov rdx, 7
	pop rcx
	imul rdx, rcx
	pop rcx
	add rdx, rcx
	push rdx
	mov edx, DWORD [rbp-16]
	push rdx
	mov edx, DWORD [rbp-20]
	pop rcx
	imul rdx, rcx
	pop rcx
	add rdx, rcx
	mov [rbx], rdx
; Expression Statement
	lea rdi, [rel LC_0]
	mov esi, DWORD [rbp-12]
	push rsi
	mov rsi, 2
	pop rcx
	add rsi, rcx
	xor rax, rax
	call _printf
	mov rax, rax

	; Return Statement
	mov rax, 0
	leave
	ret

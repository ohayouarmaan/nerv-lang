section .data
	LC_0 db "sum: %d", 10, 0
	LC_len_0 equ 9
section .text
	extern printf
	global add
	global main
add:
	push rbp
	mov rbp, rsp
	sub rsp, 16
	mov DWORD [rbp-4], edi
	mov DWORD [rbp-8], esi

	; Return Statement
	mov eax, DWORD [rbp-4]
	push rax
	mov eax, DWORD [rbp-8]
	pop rcx
	add rax, rcx
	leave
	ret
main:
	push rbp
	mov rbp, rsp
	sub rsp, 32

	; VARIABLE DECLARATION
	mov rax, 3
	mov DWORD [rbp-8], eax
	mov rax, 4
	mov DWORD [rbp-4], eax


	; VARIABLE DECLARATION
	lea rax, [rel add]
	mov QWORD [rbp-16], rax


	; VARIABLE DECLARATION

	lea rbx, [rbp-8]
	mov edi, DWORD [rbx]

	lea rbx, [rbp-8]
	add rbx, 4
	mov esi, DWORD [rbx]
	xor rax, rax
	mov rax, QWORD [rbp-16]
	call rax
	mov rax, rax
	mov DWORD [rbp-20], eax

; Expression Statement
	lea rdi, [rel LC_0]
	mov esi, DWORD [rbp-20]
	xor rax, rax
	call printf
	mov rax, rax

	; Return Statement
	mov eax, DWORD [rbp-20]
	leave
	ret

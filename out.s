section .data
section .text
	global main
	global functionA
functionA:
	push rbp
	mov rbp, rsp
	sub rsp, 8
	mov QWORD [rbp-8], rdi
	mov rax, QWORD [rbp-8]
	leave
	ret
main:
	push rbp
	mov rbp, rsp

	; VARIABLE DECLARATION
	mov rdi, 2
	sub rsp, 8
	push rdi
	mov rdi, 8
	pop rbx
	add rsp, 8
	mov rax, rbx
	xor rdx, rdx
	div rdi
	mov rdi, rax
	call functionA
	sub rsp, 8
	mov QWORD [rsp], rax

	mov eax, DWORD [rbp-8]
	leave
	ret

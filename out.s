global main
main:
	push rbp
	mov rbp, rsp
	sub rsp, 8
	mov rax, 5
	push rax
	mov rax, 3
	pop rbx
	add rsp, 8
	imul rax, rbx
	mov DWORD [rbp-4], eax
	sub rsp, 8
	mov rax, 5
	push rax
	mov eax, DWORD [rbp-4]
	pop rbx
	add rsp, 8
	add rax, rbx
	mov rax, 60
	mov rdi, rax
	syscall

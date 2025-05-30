section .data
section .text
	global main
	global functionA
functionA:
	push rbp
	mov rbp, rsp
	sub rsp, 4
	mov DWORD [rbp-4], edi
	mov eax, DWORD [rbp-4]
	leave
	ret
main:
	push rbp
	mov rbp, rsp

	; VARIABLE DECLARATION
	mov rdi, 2
	sub rsp, 8
	push rdi
	mov rdi, 2
	pop rbx
	add rsp, 8
	add rdi, rbx
	call functionA
	sub rsp, 4
	mov DWORD [rsp], eax

	mov eax, DWORD [rbp-4]
	leave
	ret

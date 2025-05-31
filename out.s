section .data
section .text
	global main
main:
	push rbp
	mov rbp, rsp

	; VARIABLE DECLARATION
	mov rax, 4
	sub rsp, 4
	mov DWORD [rsp], eax

	lea rax, [rbp-4]
	mov rbx, 2
	mov [rax], rbx
	lea rax, [rbp-4]
	mov rbx, 7
	mov [rax], rbx
	mov eax, DWORD [rbp-4]
	leave
	ret

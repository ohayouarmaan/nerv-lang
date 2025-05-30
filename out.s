section .data
section .text
	global main
	global functionA
main:
	push rbp
	mov rbp, rsp

	; VARIABLE DECLARATION
	mov rax, 3
	sub rsp, 4
	mov DWORD [rsp], eax

	mov rax, 5
	leave
	ret
functionA:
	push rbp
	mov rbp, rsp

	; VARIABLE DECLARATION
	mov rax, 2
	sub rsp, 4
	mov DWORD [rsp], eax

	mov eax, DWORD [rbp-4]
	leave
	ret

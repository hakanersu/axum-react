.PHONY: dev backend frontend

dev:
	@trap 'kill 0' INT; \
	cargo run -p ruststack-backend & \
	cd frontend && npm run dev & \
	wait

backend:
	cargo run -p ruststack-backend

frontend:
	cd frontend && npm run dev

.PHONY: dev backend frontend

dev:
	@trap 'kill 0' INT; \
	cargo run -p sekizgen-backend & \
	cd frontend && npm run dev & \
	wait

backend:
	cargo run -p sekizgen-backend

frontend:
	cd frontend && npm run dev

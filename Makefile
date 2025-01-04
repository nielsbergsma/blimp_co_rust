event_map_path := ${shell pwd}/event_map.json

install:
	cargo install -q worker-build --version 0.1.2
	cd local && npm install

build:
	cd service/scheduling_api && EVENT_MAP_PATH=$(event_map_path) worker-build --release
	cd service/reservation_api && EVENT_MAP_PATH=$(event_map_path) worker-build --release

	rm -rf local/build && mkdir local/build
	ln -s ../../service/scheduling_api/build/worker local/build/scheduling_api
	ln -s ../../service/reservation_api/build/worker local/build/reservation_api

	rm -rf deployment/dist && mkdir -p deployment/dist
	ln -s ../../service/scheduling_api/build/worker deployment/dist/scheduling_api
	ln -s ../../service/reservation_api/build/worker deployment/dist/reservation_api

serve@backend: build
	cd local && npm run serve

serve@frontend:
	cd frontend/backoffice && node server.js

deploy@scheduling-api: build
	npx --yes wrangler deploy --config deployment/scheduling-api.wrangler.toml

deploy@reservation-api: build
	npx --yes wrangler deploy --config deployment/reservation-api.wrangler.toml

deploy@backoffice:
	mkdir -p deployment/dist && cp -R frontend deployment/dist/
	sed -i -e 's|http://127.0.0.1:5000/buckets/scheduling|TODO|g' deployment/dist/frontend/backoffice/js/app.js
	npx --yes wrangler pages deploy deployment/dist/frontend/backoffice --project-name backoffice-rs
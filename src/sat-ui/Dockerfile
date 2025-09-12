FROM node:18-alpine
WORKDIR /app
COPY . .
RUN npm ci
RUN npm run build

ENTRYPOINT ["cp","-R", "/app/dist/.", "/app/dist-copy"]

FROM node:26.13.1-alpine3.24@sha256:6bd9ab43c0ec0190ca7b2d193906cb8b7edf6e7f4afa43ddeccd6452710bffb5 AS build
WORKDIR /app
COPY package.json package-lock.json ./
RUN npm install --global npm@11.21.2 && npm --version && npm ci
COPY . .
RUN npm run build

FROM nginx:1.31.8-alpine3.24@sha256:fed7e14be83cbd2661a75cdc5b872d936cf155d46c327084d30b8df19ae88d72
COPY --from=build /app/dist /usr/share/nginx/html
COPY nginx.conf /etc/nginx/conf.d/default.conf
EXPOSE 80
CMD ["nginx", "-g", "daemon off;"]

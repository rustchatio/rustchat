# Build stage
FROM node:24-alpine AS builder
WORKDIR /app
COPY package*.json ./
RUN npm cache clean --force && \
    if [ -f package-lock.json ]; then npm ci; else npm install; fi
COPY . .
RUN npm run build

# Production stage
FROM openresty/openresty:alpine

# 1. Create required directories and redirect logs to stdout/stderr
RUN mkdir -p /var/log/nginx /var/run/openresty && \
    ln -sf /dev/stdout /var/log/nginx/access.log && \
    ln -sf /dev/stderr /var/log/nginx/error.log

# 2. Copy built assets - ensure this matches the 'root' in your nginx.conf
COPY --from=builder /app/dist /usr/share/nginx/html

# 3. Copy nginx config
COPY nginx.conf /etc/nginx/conf.d/default.conf

# 4. Run as non-root user
RUN chown -R nobody:nobody /usr/share/nginx/html /var/log/nginx /var/run/openresty /etc/nginx/conf.d
USER nobody

EXPOSE 8080

CMD ["/usr/local/openresty/bin/openresty", "-g", "daemon off;"]

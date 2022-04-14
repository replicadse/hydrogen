FROM golang:1.18.1 as build-env
ARG service
WORKDIR /app
COPY ${service}/go.mod .
COPY ${service}/go.sum .
RUN go mod download
COPY ${service}/* .
RUN go build -o ./bin/app

FROM gcr.io/distroless/cc
WORKDIR /app
COPY --from=build-env /app/bin/app .
USER 1000
CMD [ "./app" ]

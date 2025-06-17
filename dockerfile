FROM python:3.11-slim

RUN apt-get update && \
    apt-get install -y cron && \
    mkdir /app

COPY . /app
WORKDIR /app

RUN crontab /app/cronjob

CMD cron -f

FROM python:3.12-slim

WORKDIR /app

COPY app.py .

EXPOSE 5000

ENV PYTHONUNBUFFERED=1 \
    PORT=5000

CMD ["python3", "app.py"]

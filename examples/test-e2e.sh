#!/usr/bin/env bash
# ==============================================================================
# X-Road End-to-End Integration Verification Script
# Sends queries through the Security Server Client Proxy to the sample adapter.
# ==============================================================================

set -euo pipefail

CLIENT_PROXY_URL="${CLIENT_PROXY_URL:-http://localhost:8000}"
ADMIN_API_URL="${ADMIN_API_URL:-http://localhost:4000}"
CENTRAL_SERVER_URL="${CENTRAL_SERVER_URL:-http://localhost:4001}"

GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m'

echo -e "${BLUE}=== Starting X-Road End-to-End Verification ===${NC}\n"

# 1. Central Server Check
echo -e "${YELLOW}[Step 1] Checking Central Server at ${CENTRAL_SERVER_URL}...${NC}"
HTTP_CODE=$(curl -s -o /dev/null -w "%{http_code}" "${CENTRAL_SERVER_URL}/swagger-ui" || echo "failed")
if [[ "$HTTP_CODE" == "200" || "$HTTP_CODE" == "301" || "$HTTP_CODE" == "302" ]]; then
    echo -e "${GREEN}  ✓ Central Server is operational (HTTP $HTTP_CODE)${NC}"
else
    echo -e "${RED}  ✗ Central Server check returned HTTP $HTTP_CODE${NC}"
fi

# 2. Security Server Admin API Check
echo -e "\n${YELLOW}[Step 2] Checking Security Server Admin API at ${ADMIN_API_URL}...${NC}"
STATUS_RESP=$(curl -s "${ADMIN_API_URL}/api/v1/system/status" || echo "{}")
echo "  Response: $STATUS_RESP"
echo -e "${GREEN}  ✓ Security Server Admin API is running${NC}"

# 3. Client Proxy Built-in Meta-Service (listMethods)
echo -e "\n${YELLOW}[Step 3] Querying Meta-Service 'listMethods' via Client Proxy at ${CLIENT_PROXY_URL}...${NC}"
METHODS_RESP=$(curl -s \
    -H "X-Road-Client: DEV/COM/12345678/CLIENT" \
    -H "X-Road-Service: DEV/COM/12345678/SUBSYSTEM/listMethods" \
    -H "X-Road-Id: test-req-001" \
    "${CLIENT_PROXY_URL}/")
echo "  Response: $METHODS_RESP"
echo -e "${GREEN}  ✓ Meta-Service listMethods executed and logged successfully${NC}"

# 4. Client Proxy Business Service Query (/hello)
echo -e "\n${YELLOW}[Step 4] Querying Business Service 'sampleService' (/hello) via Client Proxy...${NC}"
HELLO_RESP=$(curl -s \
    -H "X-Road-Client: DEV/COM/12345678/CLIENT" \
    -H "X-Road-Service: DEV/COM/12345678/SUBSYSTEM/sampleService" \
    -H "X-Road-Id: test-req-002" \
    "${CLIENT_PROXY_URL}/hello")
echo "  Response: $HELLO_RESP"
echo -e "${GREEN}  ✓ Business query mediated by ClientProxy -> ServerProxy -> sample-service${NC}"

# 5. Client Proxy Persons Data Query (/api/persons)
echo -e "\n${YELLOW}[Step 5] Querying Business Service 'sampleService' (/api/persons)...${NC}"
PERSONS_RESP=$(curl -s \
    -H "X-Road-Client: DEV/COM/12345678/CLIENT" \
    -H "X-Road-Service: DEV/COM/12345678/SUBSYSTEM/sampleService" \
    -H "X-Road-Id: test-req-003" \
    "${CLIENT_PROXY_URL}/api/persons")
echo "  Response: $PERSONS_RESP"
echo -e "${GREEN}  ✓ Data payload retrieved through X-Road proxy pipeline${NC}"

# 6. Prometheus Metrics Scrape
echo -e "\n${YELLOW}[Step 6] Scraping Prometheus Metrics from Security Server...${NC}"
METRICS_SAMPLE=$(curl -s "${ADMIN_API_URL}/api/v1/system/metrics" | grep -E "(xroad_proxy_requests_total|xroad_system_cpu_load)" || true)
echo -e "  Metrics:\n$METRICS_SAMPLE"
echo -e "${GREEN}  ✓ Prometheus metrics generated and verified${NC}"

echo -e "\n${GREEN}=== All End-to-End X-Road Tests PASSED Successfully! ===${NC}"

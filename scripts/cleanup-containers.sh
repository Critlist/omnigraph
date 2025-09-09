#!/bin/bash
# Comprehensive TestContainers cleanup script

echo "🧹 Cleaning up TestContainers and related Docker resources..."

# Stop all containers with testcontainers labels
echo "Stopping TestContainers..."
docker ps -q --filter "label=org.testcontainers=true" | xargs -r docker stop

# Remove all containers with testcontainers labels
echo "Removing TestContainers..."
docker ps -aq --filter "label=org.testcontainers=true" | xargs -r docker rm

# Stop and remove docker-compose test containers
echo "Stopping docker-compose test services..."
docker compose -f docker-compose.test.yml down -v --remove-orphans 2>/dev/null || true

# Remove any dangling volumes created by tests
echo "Removing test volumes..."
docker volume ls -q --filter "name=omnigraph" | xargs -r docker volume rm

# Clean up any dangling networks
echo "Removing test networks..."
docker network ls -q --filter "name=omnigraph-test" | xargs -r docker network rm

# Remove unused test images (optional - uncomment if desired)
# echo "Removing unused test images..."
# docker image prune -f --filter "label=org.testcontainers=true"

# Show remaining containers
echo ""
echo "📊 Remaining containers:"
docker ps -a --format "table {{.Names}}\t{{.Image}}\t{{.Status}}"

echo ""
echo "✅ Cleanup complete!"
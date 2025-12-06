#!/bin/bash
# Script para ejecutar el poblador de base de datos de Zaragoza

set -e

# Colores para output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color

echo -e "${CYAN}🗺️  Poblador de Base de Datos - Zaragoza${NC}"
echo ""

# Verificar que estamos en el directorio correcto
if [ ! -f "populate_zaragoza.py" ]; then
    echo -e "${RED}❌ Error: Este script debe ejecutarse desde el directorio auphere-places${NC}"
    echo -e "${YELLOW}   cd auphere-places${NC}"
    echo -e "${YELLOW}   ./run_populate.sh${NC}"
    exit 1
fi

# Verificar que existe el archivo .env
if [ ! -f ".env" ]; then
    echo -e "${RED}❌ Error: Archivo .env no encontrado${NC}"
    echo -e "${YELLOW}   Copia .env.example a .env y configura las variables necesarias:${NC}"
    echo -e "${YELLOW}   cp .env.example .env${NC}"
    echo -e "${YELLOW}   nano .env${NC}"
    exit 1
fi

# Verificar que las dependencias Python están instaladas
echo -e "${CYAN}🔍 Verificando dependencias Python...${NC}"
if ! python3 -c "import httpx, rich, dotenv" 2>/dev/null; then
    echo -e "${YELLOW}⚠️  Dependencias no encontradas. Instalando...${NC}"
    pip3 install -r requirements-populate.txt
    if [ $? -ne 0 ]; then
        echo -e "${RED}❌ Error al instalar dependencias${NC}"
        echo -e "${YELLOW}   Intenta manualmente: pip3 install -r requirements-populate.txt${NC}"
        exit 1
    fi
    echo -e "${GREEN}✅ Dependencias instaladas${NC}"
else
    echo -e "${GREEN}✅ Dependencias OK${NC}"
fi

# Verificar que el microservicio está corriendo
echo -e "${CYAN}🔍 Verificando que el microservicio esté corriendo...${NC}"
if ! curl -s http://localhost:3001/health > /dev/null 2>&1; then
    echo -e "${RED}❌ El microservicio no está corriendo en localhost:3001${NC}"
    echo -e "${YELLOW}   Por favor, inicia el microservicio en otra terminal:${NC}"
    echo -e "${YELLOW}   cd auphere-places${NC}"
    echo -e "${YELLOW}   cargo run${NC}"
    exit 1
fi
echo -e "${GREEN}✅ Microservicio disponible${NC}"
echo ""

# Ejecutar el script Python
echo -e "${CYAN}🚀 Ejecutando script de población...${NC}"
echo ""
python3 populate_zaragoza.py

echo ""
echo -e "${GREEN}✨ Script completado${NC}"


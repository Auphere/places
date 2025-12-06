# 🚀 Inicio Rápido - Poblar Base de Datos

## ⚡ Ejecución en 3 Pasos

### 1️⃣ Configura tu `.env`

```bash
cd auphere-places
cp .env.example .env
nano .env  # o usa tu editor favorito
```

**Variables requeridas en `.env`:**

```env
DATABASE_URL=postgresql://usuario:password@localhost:5432/places
GOOGLE_PLACES_API_KEY=tu_api_key_de_google_cloud
ADMIN_TOKEN=un_token_secreto_seguro
```

### 2️⃣ Inicia el Microservicio (Terminal 1)

```bash
cd auphere-places
cargo run
```

Espera a ver:

```
[INFO] Server running at http://127.0.0.1:3001
```

### 3️⃣ Ejecuta el Script (Terminal 2)

**Opción A - Script Bash (Recomendado):**

```bash
cd auphere-places
./run_populate.sh
```

**Opción B - Python Directo:**

```bash
cd auphere-places
pip3 install -r requirements-populate.txt
python3 populate_zaragoza.py
```

---

## ⏱️ ¿Cuánto Tardará?

- **Primera ejecución**: 15-20 minutos
- **Lugares creados**: ~700-800 lugares
- **Costo Google API**: ~$6-7 USD

## ✅ Verificación

```bash
# Ver lugares creados
curl "http://localhost:3001/places/search?city=Zaragoza&limit=5"

# Ver estadísticas
curl http://localhost:3001/admin/stats -H "X-Admin-Token: tu_token_aqui"
```

## 🆘 ¿Problemas?

Lee `POPULATE_GUIDE.md` para guía completa y troubleshooting.

---

**¡Listo!** Ahora tienes ~700-800 lugares de Zaragoza en tu base de datos 🎉


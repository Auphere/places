#!/usr/bin/env python3
"""
Script para poblar la base de datos con lugares de Zaragoza.

Este script utiliza el endpoint de sincronización del microservicio auphere-places
para poblar la base de datos con diferentes tipos de lugares de ocio y entretenimiento
basados en la documentación oficial de Google Places API (Tabla A y Tabla B).

Referencia: https://developers.google.com/maps/documentation/places/web-service/place-types

Uso:
    python populate_zaragoza.py

Requisitos:
    - El microservicio auphere-places debe estar corriendo en localhost:3001
    - Tener configurado GOOGLE_PLACES_API_KEY en el .env
    - Tener configurado ADMIN_TOKEN en el .env
"""

import os
import sys
import time
from datetime import datetime
from pathlib import Path
from typing import Dict, List, Optional

import httpx
from dotenv import load_dotenv
from rich.console import Console
from rich.progress import Progress, SpinnerColumn, TextColumn, BarColumn, TaskProgressColumn
from rich.table import Table
from rich.panel import Panel
from rich import box

# Configuración
console = Console()

# Tipos de lugares a sincronizar
# Basado en: https://developers.google.com/maps/documentation/places/web-service/place-types
# 
# Tabla A - Ocio y entretenimiento:
# - amusement_center, amusement_park, aquarium, banquet_hall, bowling_alley, casino, 
# - community_center, convention_center, cultural_center, dog_park, event_venue,
# - hiking_area, historical_landmark, marina, movie_rental, movie_theater, 
# - national_park, night_club, park, tourist_attraction, visitor_center, wedding_venue, zoo
#
# Tabla A - Comidas y bebidas:
# - american_restaurant, bakery, bar, barbecue_restaurant, brazilian_restaurant,
# - breakfast_restaurant, brunch_restaurant, cafe, chinese_restaurant, coffee_shop,
# - fast_food_restaurant, french_restaurant, greek_restaurant, hamburger_restaurant,
# - ice_cream_shop, indian_restaurant, indonesian_restaurant, italian_restaurant,
# - japanese_restaurant, korean_restaurant, lebanese_restaurant, meal_delivery,
# - meal_takeaway, mediterranean_restaurant, mexican_restaurant, middle_eastern_restaurant,
# - pizza_restaurant, ramen_restaurant, restaurant, sandwich_shop, seafood_restaurant,
# - spanish_restaurant, steak_house, sushi_restaurant, thai_restaurant, turkish_restaurant,
# - vegan_restaurant, vegetarian_restaurant, vietnamese_restaurant
#
# Tabla B - Valores adicionales:
# - point_of_interest

PLACE_TYPES = {

    # === TABLA B - Valores Adicionales ===
    "point_of_interest": {
        "name_es": "Puntos de Interés",
        "icon": "📍",
        "cell_size_km": 2.0,
        "radius_m": 1500,
    },
}


class PlacesSyncManager:
    """Gestor de sincronización de lugares."""

    def __init__(
        self,
        base_url: str = "http://localhost:3001",
        admin_token: Optional[str] = None,
        timeout: int = 300,
    ):
        """
        Inicializa el gestor de sincronización.

        Args:
            base_url: URL base del microservicio auphere-places
            admin_token: Token de administración para autenticación
            timeout: Timeout en segundos para cada request
        """
        self.base_url = base_url
        self.admin_token = admin_token
        self.timeout = timeout
        self.results: List[Dict] = []

    def check_service_health(self) -> bool:
        """
        Verifica que el servicio esté disponible.

        Returns:
            True si el servicio está disponible, False en caso contrario
        """
        try:
            with httpx.Client(timeout=5.0) as client:
                response = client.get(f"{self.base_url}/health")
                if response.status_code == 200:
                    return True
                return False
        except Exception as e:
            console.print(f"[red]Error al conectar con el servicio: {e}[/red]")
            return False

    def sync_place_type(
        self,
        place_type: str,
        city: str = "Zaragoza",
        cell_size_km: float = 2.5,
        radius_m: int = 1000,
    ) -> Dict:
        """
        Sincroniza un tipo de lugar específico.

        Args:
            place_type: Tipo de lugar (restaurant, bar, cafe, etc.)
            city: Ciudad a sincronizar
            cell_size_km: Tamaño de celda del grid en km
            radius_m: Radio de búsqueda en metros

        Returns:
            Diccionario con resultados de la sincronización
        """
        url = f"{self.base_url}/admin/sync/{city}"
        headers = {
            "X-Admin-Token": self.admin_token,
            "Content-Type": "application/json",
        }
        payload = {
            "place_type": place_type,
            "cell_size_km": cell_size_km,
            "radius_m": radius_m,
        }

        try:
            with httpx.Client(timeout=self.timeout) as client:
                response = client.post(url, json=payload, headers=headers)
                response.raise_for_status()
                return response.json()
        except httpx.HTTPStatusError as e:
            error_detail = "Unknown error"
            try:
                error_detail = e.response.json().get("error", str(e))
            except:
                error_detail = e.response.text or str(e)

            return {
                "success": False,
                "error": error_detail,
                "status_code": e.response.status_code,
            }
        except Exception as e:
            return {"success": False, "error": str(e)}

    def get_stats(self) -> Optional[Dict]:
        """
        Obtiene estadísticas de la base de datos.

        Returns:
            Diccionario con estadísticas o None si hay error
        """
        url = f"{self.base_url}/admin/stats"
        headers = {"X-Admin-Token": self.admin_token}

        try:
            with httpx.Client(timeout=10.0) as client:
                response = client.get(url, headers=headers)
                response.raise_for_status()
                return response.json()
        except Exception as e:
            console.print(f"[yellow]No se pudieron obtener estadísticas: {e}[/yellow]")
            return None

    def run_full_sync(self, place_types: Optional[List[str]] = None) -> None:
        """
        Ejecuta sincronización completa de todos los tipos de lugares.

        Args:
            place_types: Lista de tipos de lugares a sincronizar (None = todos)
        """
        # Verificar salud del servicio
        console.print("\n[cyan]🔍 Verificando estado del servicio...[/cyan]")
        if not self.check_service_health():
            console.print("[red]❌ El servicio no está disponible.[/red]")
            console.print(
                "[yellow]Por favor, asegúrate de que auphere-places esté corriendo:[/yellow]"
            )
            console.print("   cd auphere-places")
            console.print("   cargo run")
            sys.exit(1)

        console.print("[green]✅ Servicio disponible[/green]\n")

        # Filtrar tipos si se especificaron
        types_to_sync = PLACE_TYPES
        if place_types:
            types_to_sync = {
                k: v for k, v in PLACE_TYPES.items() if k in place_types
            }

        # Mostrar resumen
        self._print_header(types_to_sync)

        # Obtener estadísticas iniciales
        console.print("[cyan]📊 Estadísticas iniciales:[/cyan]")
        initial_stats = self.get_stats()
        if initial_stats:
            self._print_stats_table(initial_stats, "Antes de sincronización")

        # Sincronizar cada tipo de lugar
        console.print(f"\n[bold cyan]🚀 Iniciando sincronización...[/bold cyan]\n")

        total_places_created = 0
        total_places_skipped = 0
        total_api_requests = 0
        total_duration = 0

        with Progress(
            SpinnerColumn(),
            TextColumn("[progress.description]{task.description}"),
            BarColumn(),
            TaskProgressColumn(),
            console=console,
        ) as progress:

            main_task = progress.add_task(
                "[cyan]Sincronizando tipos de lugares...",
                total=len(types_to_sync),
            )

            for place_type, config in types_to_sync.items():
                type_name = config["name_es"]
                icon = config["icon"]

                progress.update(
                    main_task,
                    description=f"[cyan]Sincronizando {icon} {type_name}...",
                )

                start_time = time.time()
                result = self.sync_place_type(
                    place_type=place_type,
                    cell_size_km=config["cell_size_km"],
                    radius_m=config["radius_m"],
                )
                duration = time.time() - start_time

                # Procesar resultado
                if result.get("success") is False:
                    console.print(
                        f"[red]❌ Error al sincronizar {type_name}: {result.get('error')}[/red]"
                    )
                    self.results.append(
                        {
                            "type": place_type,
                            "type_name": type_name,
                            "icon": icon,
                            "success": False,
                            "error": result.get("error"),
                            "duration": duration,
                        }
                    )
                else:
                    places_created = result.get("places_created", 0)
                    places_skipped = result.get("places_skipped", 0)
                    api_requests = result.get("api_requests", 0)

                    total_places_created += places_created
                    total_places_skipped += places_skipped
                    total_api_requests += api_requests
                    total_duration += duration

                    console.print(
                        f"[green]✅ {icon} {type_name}: {places_created} nuevos, "
                        f"{places_skipped} duplicados ({duration:.1f}s)[/green]"
                    )

                    self.results.append(
                        {
                            "type": place_type,
                            "type_name": type_name,
                            "icon": icon,
                            "success": True,
                            "places_created": places_created,
                            "places_skipped": places_skipped,
                            "api_requests": api_requests,
                            "duration": duration,
                        }
                    )

                progress.advance(main_task)

                # Pequeña pausa entre sincronizaciones para no saturar la API
                time.sleep(1)

        # Obtener estadísticas finales
        console.print("\n[cyan]📊 Estadísticas finales:[/cyan]")
        final_stats = self.get_stats()
        if final_stats:
            self._print_stats_table(final_stats, "Después de sincronización")

        # Mostrar resumen final
        self._print_summary(
            total_places_created,
            total_places_skipped,
            total_api_requests,
            total_duration,
        )

    def _print_header(self, types_to_sync: Dict) -> None:
        """Imprime el encabezado del script."""
        header_text = """
[bold cyan]🗺️  Poblador de Base de Datos - Zaragoza (Ocio y Entretenimiento)[/bold cyan]

Este script sincronizará los siguientes tipos de lugares desde Google Places API:
Basado en Tabla A (Ocio y entretenimiento + Comidas y bebidas) y Tabla B (point_of_interest)
"""
        console.print(Panel(header_text.strip(), box=box.ROUNDED))

        # Contar categorías
        ocio_count = 23  # amusement_center hasta zoo
        comida_count = 41  # american_restaurant hasta vietnamese_restaurant
        adicional_count = 1  # point_of_interest
        
        console.print(f"\n[bold]📊 Total de tipos a sincronizar: {len(types_to_sync)}[/bold]")
        console.print(f"  🎭 Ocio y entretenimiento: {ocio_count} tipos")
        console.print(f"  🍽️  Comidas y bebidas: {comida_count} tipos")
        console.print(f"  📍 Puntos de interés: {adicional_count} tipo")
        console.print()

    def _print_stats_table(self, stats: Dict, title: str) -> None:
        """Imprime una tabla con las estadísticas."""
        table = Table(title=title, box=box.SIMPLE)
        table.add_column("Métrica", style="cyan")
        table.add_column("Valor", style="green", justify="right")

        # Estadísticas por tipo
        if "places_by_type" in stats:
            for type_stat in stats["places_by_type"]:
                table.add_row(
                    f"  {type_stat['type']}",
                    str(type_stat["count"]),
                )

        # Estadísticas por ciudad
        if "places_by_city" in stats:
            table.add_row("", "")  # Separador
            for city_stat in stats["places_by_city"]:
                table.add_row(
                    f"📍 {city_stat['city']}",
                    str(city_stat["count"]),
                )

        # Rating promedio
        if "average_rating" in stats and stats["average_rating"] is not None:
            table.add_row("", "")  # Separador
            table.add_row("⭐ Rating Promedio", f"{stats['average_rating']:.2f}")

        console.print(table)
        console.print()

    def _print_summary(
        self,
        total_created: int,
        total_skipped: int,
        total_requests: int,
        total_duration: float,
    ) -> None:
        """Imprime el resumen final."""
        # Tabla de resumen por tipo
        summary_table = Table(
            title="📋 Resumen de Sincronización",
            box=box.ROUNDED,
            show_header=True,
            header_style="bold cyan",
        )
        summary_table.add_column("Tipo", style="cyan")
        summary_table.add_column("Estado", justify="center")
        summary_table.add_column("Nuevos", justify="right", style="green")
        summary_table.add_column("Duplicados", justify="right", style="yellow")
        summary_table.add_column("Requests", justify="right", style="blue")
        summary_table.add_column("Duración", justify="right", style="magenta")

        for result in self.results:
            if result["success"]:
                summary_table.add_row(
                    f"{result['icon']} {result['type_name']}",
                    "✅",
                    str(result["places_created"]),
                    str(result["places_skipped"]),
                    str(result["api_requests"]),
                    f"{result['duration']:.1f}s",
                )
            else:
                summary_table.add_row(
                    f"{result['icon']} {result['type_name']}",
                    "❌",
                    "-",
                    "-",
                    "-",
                    f"{result['duration']:.1f}s",
                )

        console.print("\n")
        console.print(summary_table)

        # Resumen total
        total_summary = f"""
[bold green]✨ Sincronización Completada[/bold green]

📊 Totales:
  • Lugares nuevos creados: [bold green]{total_created}[/bold green]
  • Lugares duplicados (omitidos): [bold yellow]{total_skipped}[/bold yellow]
  • Requests a Google Places API: [bold blue]{total_requests}[/bold blue]
  • Duración total: [bold magenta]{total_duration:.1f}s ({total_duration/60:.1f} min)[/bold magenta]

💰 Costo estimado: [bold]${total_requests * 0.017:.2f} USD[/bold]
   (basado en $0.017 por request a Google Places API)
"""
        console.print(Panel(total_summary.strip(), box=box.DOUBLE, border_style="green"))


def main():
    """Función principal."""
    # Cargar variables de entorno
    env_path = Path(__file__).parent / ".env"
    if env_path.exists():
        load_dotenv(env_path)
        console.print(f"[green]✅ Variables de entorno cargadas desde {env_path}[/green]")
    else:
        console.print(
            f"[yellow]⚠️  Archivo .env no encontrado en {env_path}[/yellow]"
        )
        console.print("[yellow]Asegúrate de tener configuradas las variables de entorno[/yellow]")

    # Obtener configuración
    admin_token = os.getenv("ADMIN_TOKEN")
    if not admin_token:
        console.print("[red]❌ Error: ADMIN_TOKEN no está configurado en .env[/red]")
        console.print("[yellow]Por favor, configura ADMIN_TOKEN en tu archivo .env[/yellow]")
        sys.exit(1)

    google_api_key = os.getenv("GOOGLE_PLACES_API_KEY")
    if not google_api_key:
        console.print(
            "[red]❌ Error: GOOGLE_PLACES_API_KEY no está configurado en .env[/red]"
        )
        console.print("[yellow]Por favor, configura GOOGLE_PLACES_API_KEY en tu archivo .env[/yellow]")
        sys.exit(1)

    base_url = os.getenv("PLACES_API_URL", "http://localhost:3001")

    # Crear gestor y ejecutar sincronización
    manager = PlacesSyncManager(base_url=base_url, admin_token=admin_token)

    try:
        manager.run_full_sync()

        # Mensaje final
        console.print("\n[bold green]🎉 ¡Proceso completado exitosamente![/bold green]")
        console.print("\n[cyan]Próximos pasos:[/cyan]")
        console.print("  1. Verificar los datos: [bold]curl http://localhost:3001/places/search?city=Zaragoza[/bold]")
        console.print("  2. Ver estadísticas: [bold]curl http://localhost:3001/admin/stats -H 'X-Admin-Token: <token>'[/bold]")
        console.print("  3. Probar búsquedas en tu aplicación frontend/agent")
        console.print()

    except KeyboardInterrupt:
        console.print("\n[yellow]⚠️  Sincronización interrumpida por el usuario[/yellow]")
        sys.exit(1)
    except Exception as e:
        console.print(f"\n[red]❌ Error inesperado: {e}[/red]")
        import traceback
        console.print(f"[red]{traceback.format_exc()}[/red]")
        sys.exit(1)


if __name__ == "__main__":
    main()

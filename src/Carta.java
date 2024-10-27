import java.util.HashMap;
import java.util.List;
import java.util.Map;

public class Carta {
	
	private char palo;
	private int numero;
	
	
	public Carta( char palo, int numero) {
		this.palo = palo;
		if ( numero == 1) {
			numero = 14;	// Para facilitar cálculos con el AS, ESCALERAS Y CARTAS ALTAS
		}
		this.numero = numero;
	}
	
	public char getPalo() {
		return this.palo;
	}
	
	public int getNumero() {
		return this.numero;
	}
	
	public boolean masAltaQue( Carta carta ) {
		return this.getNumero() > carta.getNumero();
	}
	
	public boolean mismoPaloQue( Carta carta ) {
		return this.getPalo() == carta.getPalo();
	}
	
	public boolean mismoNumeroQue( Carta carta ) {
		return this.getNumero() == carta.getNumero();
	}
	
	public boolean puedenHacerEscalera( Carta carta ) {
		int diferencia = this.getNumero() - carta.getNumero();
		return diferencia <= 4 && diferencia >= -4;
	}
	
	/*
	 * Esta función nos ayudará a saber si hay cartas que aparecen n veces
	 * 
	 * Devolverá: 
	 * 	-1 --> No hay ninguna carta que se repita tantas veces
	 *   c --> Siendo c un entero igual al numero de la carta que se repite n veces
	 */
	static int hay_N_CartasRepetidasMismoNumero( List<Carta> cartas , int n) {
		
		Map< Integer , Integer> contarCartas = new HashMap<>();

        for (Carta c : cartas) {
        	int numeroC = c.getNumero();
            int count = contarCartas.getOrDefault( numeroC, 0) + 1;
            if (count >= n) {
                return numeroC; // Detenernos en cuanto encontramos un elemento con n apariciones
            }
            contarCartas.put(numeroC, count);
        }
        
        // Si no encontramos ningún elemento con exactamente 'n' apariciones
        return -1;
	}
	
	static int hay_N_CartasRepetidasMismoNumeroMesa( List<Carta> cartas , int n) {
		
		Map< Integer , Integer> contarCartas = new HashMap<>();

        for ( int i = 2 ; i < cartas.size() ; ++i ) {
        	int numeroC = cartas.get(i).getNumero();
            int count = contarCartas.getOrDefault( numeroC, 0) + 1;
            if (count >= n) {
                return numeroC; // Detenernos en cuanto encontramos un elemento con n apariciones
            }
            contarCartas.put(numeroC, count);
        }
        
        // Si no encontramos ningún elemento con exactamente 'n' apariciones
        return -1;
	}
	
	/*
	 * 	Devuelve el número de veces que se repite ese número enntre la mesa y nuestra mano
	 */
	static int vecesQueSeSaleEsteNumero( List<Carta> cartas , int numCarta ) {
		
		Map< Integer , Integer> contarCartas = new HashMap<>();

        for (Carta c : cartas) {
        	int numeroC = c.getNumero();
            int count = contarCartas.getOrDefault( numeroC, 0) + 1;
            contarCartas.put(numeroC, count);
        }
        
        return contarCartas.get( numCarta );
	}
	
}

import java.util.List;

public class Carta {
	
	private char palo;
	private int numero;
	
	
	public Carta( char palo, int numero) {
		this.palo = palo;
		if ( numero == 1) {
			numero = 14;	// Para facilitar los cálculos del AS
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
	
	static boolean hayParejas( List<Carta> cartas ) {
		
		boolean hayParejas = false;
		for( int i = 2 ; (i < cartas.size() - 1 ) && !hayParejas ; ++i) {
			hayParejas = cartas.get(i).mismoNumeroQue( cartas.get(i+1) );
		}
		return hayParejas;
	}
}

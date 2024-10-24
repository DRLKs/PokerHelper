
public class CalculoDeProbabilidades {
	
	public int C( int n1 ,int n2) { // COMBINACIONES
		
		if( n1 < n2 ) { // ERROR
			return 0;
		}
		
		int n3 = n1 - n2;
		int totalNumerador = 1;
		for( int i = n3 + 1 ; i <= n1 ; ++i ) {
			totalNumerador *= i;
		}
		int totalDenominador = 1;
		for( int i = 2 ; i <= n2 ; ++i ) {
			totalDenominador *= i;
		}
		return totalNumerador / totalDenominador;
	}
	
	/* 
	 * numCartas****, 	son las cartas de **** que tienes en tu mano
	 * ronda,			es la ronda, parte de la mano en la que nos encontramos
	 * 		0 = Solo tenemos nuestras cartas
	 * 		1 = Tenemos nuestras cartas y 3 más en la mesa (FLOP)
	 * 		2 = Tenemos nuestras cartas y 4 más	(TURN)
	 * 		3 = Tenemos todas las cartas  (RIVER)
	 * 
	 * 	CABE DESTACAR: El número de jugadores no interviene en las probabilidades que haya de obtener tu mano deseada
	 * 
	 * */
	public double completarColor( int numCartasColor, int ronda ){
		
		double prob = 0.0;
		int cartasNecesarias = 5 - numCartasColor;
		int cartasEseTipoRestantes = 12 - numCartasColor;
		
		if( cartasNecesarias <=0 ) {
			prob = 1;
		}else if( ronda < 3 ){
	
			int cartasQueFaltanPorConocer = 0;
			if( ronda == 0) {
				cartasQueFaltanPorConocer = 50;
				prob = 1.0;
				
			}else if( ronda == 1 && cartasNecesarias <= 2) {
				cartasQueFaltanPorConocer = 47;
				prob = 1.0;
			
			}else if( ronda == 2 && cartasNecesarias == 1 ) {
				cartasQueFaltanPorConocer = 46;
				prob = 1.0;
			}
			
			while( cartasNecesarias > 0 ) {
				prob *= (double ) cartasEseTipoRestantes / cartasQueFaltanPorConocer;
				--cartasEseTipoRestantes;
				--cartasQueFaltanPorConocer;
				--cartasNecesarias;
			}
			
		}
		return prob;
	}
	
	public double completarMano( int numCartasMano, int ronda ){
		
		double prob = 0.0;
		int cartasNecesarias = 5 - numCartasMano;
		int cartasEseTipoRestantes = 12 - numCartasMano;
		
		if( cartasNecesarias <=0 ) {
			prob = 1.0;
		}else if( ronda < 3 ){
	
			int cartasQueFaltanPorConocer = 0;
			
			
			if( ronda == 0) {
				cartasQueFaltanPorConocer = 50;
				prob = 1.0;
				
			}else if( ronda == 1 ) {
				cartasQueFaltanPorConocer = 47;
				prob = 1.0;
			
			}else if( ronda == 2 ) {
				cartasQueFaltanPorConocer = 46;
				prob = 1.0;
			}
			
			while( cartasNecesarias > 0 ) {
				prob *= (double ) cartasEseTipoRestantes / cartasQueFaltanPorConocer;
				cartasEseTipoRestantes -= 4;
				--cartasQueFaltanPorConocer;
				--cartasNecesarias;
			}
			
		}
		return prob;
	}
	
		
}

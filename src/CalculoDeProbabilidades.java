import java.util.List;

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
	 * Estas Funciones calcularán las posibilidades de obtener color
	 * numCartasColor: 	son las cartas de **** que tienes en tu mano
	 * ronda:			es la ronda, parte de la mano en la que nos encontramos
	 * 		0 = Solo tenemos nuestras cartas
	 * 		1 = Tenemos nuestras cartas y 3 más en la mesa (FLOP)
	 * 		2 = Tenemos nuestras cartas y 4 más	(TURN)
	 * 		3 = Tenemos todas las cartas  (RIVER)
	 * 
	 * 	CABE DESTACAR: El número de jugadores no interviene en las probabilidades que haya de obtener tu mano deseada
	 * 
	 * */
	public double completarColor( List<Carta> cartas ){
		
		double prob = 0.0;
		int numCartasColor = 1;
		int numCartas = cartas.size();
		int ronda = 0;
		if( numCartas == 5 ) {
			ronda = 1;
		}else if( numCartas == 6 ) {
			ronda = 2;
		}else if( numCartas == 7 ){
			ronda = 3;
		}
		
		if( cartas.get(0).mismoPaloQue( cartas.get(1) ) ) {
			++numCartasColor;
			
			for( int i = 2; i < numCartas ; ++i ) {
				if( cartas.get(i).mismoPaloQue(cartas.get(0)) ) {
					++numCartasColor;
				}
			}
			
			prob = probCompletarColor(numCartasColor, ronda );
		}else {
			
			for( int idx = 0; idx < 2 ; ++idx ) {
				for( int i = 2; i < numCartas ; ++i ) {
					if( cartas.get(i).mismoPaloQue(cartas.get(idx)) ) {
						++numCartasColor;
					}
				}
			prob += probCompletarColor(numCartasColor, ronda);
			numCartasColor = 1;
			}
		
		}

		return prob;
	}
	
	private double probCompletarColor( int numCartasColor, int ronda ) {
		
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
	
	public double completarEscalera( List<Carta> cartas ){
		
		double prob;
		int ronda = 0;
		int numCartas = cartas.size();
		if( numCartas == 5 ) {
			ronda = 1;
		}else if( numCartas == 6 ) {
			ronda = 2;
		}else if( numCartas == 7 ){
			ronda = 3;
		}
		int numCartasEscalera = 1;
		
		if( cartas.get(0).mismoPaloQue( cartas.get(1) ) ) {
			++numCartasEscalera;
			
			for( int i = 2; i < numCartas ; ++i ) {
				if( cartas.get(i).mismoPaloQue(cartas.get(0)) ) {
					++numCartasEscalera;
				}
			}
			
			prob = probCompletarEscalera(numCartasEscalera, ronda );
		}else {
			prob = 0.0;
			for( int idx = 0; idx < 2 ; ++idx ) {
				for( int i = 2; i < numCartas ; ++i ) {
					if( cartas.get(i).puedenHacerEscalera(cartas.get(idx)) ) {
						++numCartasEscalera;
					}
				}
			prob += probCompletarEscalera(numCartasEscalera, ronda);
			numCartasEscalera = 1;
			}
			
		}
		return prob;
	}
	
	private double probCompletarEscalera( int numCartasEscalera, int ronda ){
		
		double prob = 0.0;

		int cartasNecesarias = 5 - numCartasEscalera;
		int cartasEseTipoRestantes = 12 - numCartasEscalera;
		
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

	
	public double completarFullHouse( int numCartasEscalera, int ronda ){
		
		double prob = 0.0;
		int cartasNecesarias = 5 - numCartasEscalera;
		int cartasEseTipoRestantes = 12 - numCartasEscalera;
		
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

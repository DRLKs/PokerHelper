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
	
			int cartasQueNoHanSalido;
			if( ronda == 0) {
				cartasQueNoHanSalido = 50;
				
			}else if( ronda == 1 && cartasNecesarias <= 2) {
				cartasQueNoHanSalido = 47;
			
			}else if( ronda == 2 && cartasNecesarias == 1 ) {
				cartasQueNoHanSalido = 46;
			}else {
				return 0;	
			}
			prob = 1.0;
			while( cartasNecesarias > 0 ) {
				prob *= (double ) cartasEseTipoRestantes / cartasQueNoHanSalido;
				--cartasEseTipoRestantes;
				--cartasQueNoHanSalido;
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
		int cartasEseTipoRestantes = 4 * cartasNecesarias;
		
		if( cartasNecesarias <=0 ) {
			prob = 1.0;
		}else if( ronda < 3 ){
	
			int cartasQueNoHanSalido;
			if( ronda == 0) {
				cartasQueNoHanSalido = 50;
				
			}else if( ronda == 1 && cartasNecesarias <= 2) {
				cartasQueNoHanSalido = 47;
			
			}else if( ronda == 2 && cartasNecesarias == 1 ) {
				cartasQueNoHanSalido = 46;
			}else {
				return 0;	
			}
			
			prob = 1.0;
			while( cartasNecesarias > 0 ) {
				prob *= (double ) cartasEseTipoRestantes / cartasQueNoHanSalido;
				cartasEseTipoRestantes -= 4;
				--cartasQueNoHanSalido;
				--cartasNecesarias;
			}
			
		}
		return prob;
	}

	public double completarFullHouse( List<Carta> cartas ) {
		
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
		
		if( cartas.get(0).mismoNumeroQue( cartas.get(1) ) ) {	// Comprobamos que nuestras cartas tengan el mismo número	
			if( ronda == 0 ) {
				prob = probCompletarFullHouse(2, 0, ronda) * probCompletarFullHouse(0, 2, ronda);
			}else{
				int numCartaTrio = Carta.hay_N_CartasRepetidasMismoNumero(cartas, 3);
				
			}
		}else {	// Nuestas cartas no son parejas entre ellas
			if( ronda == 0 ) {
				prob = probCompletarFullHouse(1, 0, ronda) * probCompletarFullHouse(0, 1, ronda);
			}else {
				
			}
		}
		
		
		return prob;
	}
	
	private double probCompletarFullHouse( int numCartasTrio, int numCartasPareja, int ronda ){
		
		double prob = 0.0;
		int cartasNecesariasTrio = 3 - numCartasTrio;
		int cartasNecesariasPareja = 2 - numCartasPareja;

		
		if( cartasNecesariasTrio <= 0 && cartasNecesariasPareja <= 0) {
			prob = 1.0;
		}else if( ronda < 3 ){
	
			int cartasQueNoHanSalido;
			if( ronda == 0) {
				cartasQueNoHanSalido = 50;
				
			}else if( ronda == 1 && (cartasNecesariasTrio + cartasNecesariasPareja) <= 2) {
				cartasQueNoHanSalido = 47;
			
			}else if( ronda == 2 && (cartasNecesariasTrio + cartasNecesariasPareja) == 1 ) {
				cartasQueNoHanSalido = 46;
			}else {
				return 0;	
			}
			
			prob = 1.0;
			while( cartasNecesariasTrio > 0 ) {
				prob *= (double ) cartasQueNoHanSalido / cartasQueNoHanSalido;
				cartasQueNoHanSalido -= 4;
				--cartasNecesariasTrio;
				--cartasQueNoHanSalido;
			}
			
			while( cartasNecesariasPareja > 0 ) {
				prob *= (double ) cartasQueNoHanSalido / cartasQueNoHanSalido;
				cartasQueNoHanSalido -= 4;
				--cartasNecesariasPareja;
				--cartasQueNoHanSalido;
			}
			
		}
		return prob;
	}
	
		
}

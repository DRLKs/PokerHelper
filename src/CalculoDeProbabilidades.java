/*
 * MADE BY DRLK
 */
import java.io.IOException;
import java.io.PrintWriter;
import java.net.ServerSocket;
import java.net.Socket;
import java.util.List;

public class CalculoDeProbabilidades {
	/*
	 * Todas las probabilidades son sobre 1
	 */
	private double probEscalera;
	private double probColor;
	private double probFullHouse;
	private double probPoker;
	private double probEscaleraColor;
	private double probEscaleraReal;
	
	private double probEscaleraCont;
	private double probColorCont;
	private double probFullHouseCont;
	private double probPokerCont;
	private double probEscaleraColorCont;
	private double probEscaleraRealCont;
	
	private final int MAX_CARTAS_VISIBLES = 7;
	private final int IDX_CARTA_MANO_1 = 0;
	private final int IDX_CARTA_MANO_2 = 1;
	private final int NUM_CARTAS_NUNCA_VES = 45;

	/*
	 * Recalcula toda la información 
	 * Se llamará cuando se actualicen los datos
	 * Esta función manda la información al programa predictor mediante Sockets
	 */
	public void reiniciarDatos( List<Carta> cartas ) {
		probEscalera = completarEscalera(cartas);
		probColor = completarColor(cartas);
		probFullHouse = completarFullHouse(cartas);
		probPoker = completarPoker(cartas);
		probEscaleraColor = completarEscaleraColor(cartas);
		probEscaleraReal = completarEscaleraReal(cartas);
		
		/*
		try (ServerSocket serverSocket = new ServerSocket(5000)) {
            System.out.println("Esperando conexión de Python...");
            Socket clientSocket = serverSocket.accept();
            System.out.println("Conexión establecida con Python");


            // Enviar los datos a Python
            PrintWriter out = new PrintWriter(clientSocket.getOutputStream(), true);
            // MIAS
            out.println(probEscalera);
            out.println(probColor);
            out.println(probFullHouse);
            out.println(probPoker);
            out.println(probEscaleraColor);
            out.println(probEscaleraReal);
            // DE LOS CONTRINCANTES
            out.println(probEscaleraCont);
            out.println(probColorCont);
            out.println(probFullHouseCont);
            out.println(probPokerCont);
            out.println(probEscaleraColorCont);
            out.println(probEscaleraRealCont);

            System.out.println("Datos enviados a Python");

            // Cerrar la conexión
            out.close();
            clientSocket.close();
            serverSocket.close();
        } catch (IOException e) {
            e.printStackTrace();
        }
        */
    }
	
	public double getProbEscalera() {
		return probEscalera;
	}

	public double getProbColor() {
		return probColor;
	}

	public double getProbFullHouse() {
		return probFullHouse;
	}

	public double getProbPoker() {
		return probPoker;
	}

	public double getProbEscaleraColor() {
		return probEscaleraColor;
	}

	public double getProbEscaleraReal() {
		return probEscaleraReal;
	}
	
	public int C( int n ,int k) { // COMBINACIONES
		
		if (k > n || k < 0) return 0; // No existe combinatoria si k > n
        if (k == 0 || k == n ) return 1;
        // La combinación es simétrica, C(n, k) = C(n, n-k)
        if (k > n - k) {
            k = n - k;
        }
        int ini = 1 + n - k; 
        int resultado = 1;
        for (int i = ini; i <= n; ++i) {
            resultado *= i;
        }
        for(int i = 2 ; i <= k ; ++i) {
        	resultado /= i;
        }

        return resultado;
	}
	
	/* 
	 * Estas Funciones calcularán las posibilidades de obtener color
	 * 
	 * 	CABE DESTACAR: El número de jugadores no interviene en las probabilidades que haya de obtener tu mano deseada
	 * 
	 * */
	private double completarColor( List<Carta> cartas ){
		
		double prob;
		int numCartasColor = 1;
		int numCartas = cartas.size();
		int cartasPorMostrar = 5 - (numCartas - 2);
		int numCartasNecesarias;
		if( cartas.get(0).mismoPaloQue( cartas.get(1) ) ) {
			++numCartasColor;
			
			for( int i = 2; i < numCartas ; ++i ) {
				if( cartas.get(i).mismoPaloQue(cartas.get(0)) ) {
					++numCartasColor;
				}
			}
			numCartasNecesarias = 5-numCartasColor;
			prob = distribucionHiperGeometrica(numCartasNecesarias, 13-numCartasColor, cartasPorMostrar);
		}else {
			
			prob = 0.0;
			for( int idxCMano = 0 ; idxCMano < 2 ; ++idxCMano ) {	
					for( int i = 2; i < numCartas ; ++i ) {
						if( cartas.get(i).mismoPaloQue(cartas.get(idxCMano)) ) {
							++numCartasColor;
					}
				}
				numCartasNecesarias = 5 - numCartasColor;
				prob += distribucionHiperGeometrica(numCartasNecesarias, 13-numCartasColor, cartasPorMostrar);
				numCartasColor = 1;
			}
		}

		return prob;
	}
	
	/*
	 * El conjunto de las siguiente funciones calcula la probabilidad de que nuestras cartas hagan escalera normal
	 */
	private double completarEscalera( List<Carta> cartas ){
			
		double prob = 0.0;
		int numCartas = cartas.size();
		int cartasPorMostrar = MAX_CARTAS_VISIBLES - numCartas;
		int numCartaMano1 = cartas.get(IDX_CARTA_MANO_1).getNumero();
		int numCartaMano2 = cartas.get(IDX_CARTA_MANO_2).getNumero();
		int arrayInicio;
		int arrayFinal;
		
		/*
		 * El array "numerosEscalera" tendrá un tamaño igual a 14
		 *  En este array podremos comprobar si tenemos un número, por ejemplo:
		 *  
		 *  Comprobamos que tenemos el AS -> numerosEscalera[14] == 1
		 *  		TRUE -> Tenemos el AS
		 *  		FALSE -> No tenemos el AS
		 *  El AS al tener asociado el número 14, lo podremos comprobar en el lugar 14 y a la 1
		 *  No usaremos la posición 0
		 */
		int[] numerosEscalera;	
		int[] arrayFronterasEscalera;
		
		// Nuestras 2 cartas pueden hacer escalera
		if( cartas.get(0).puedenHacerEscalera( cartas.get(1) )) {	
			numerosEscalera = new int[15];
			
			numerosEscalera[ numCartaMano1 ] = 1;
			numerosEscalera[ numCartaMano2 ] = 1;
			
			arrayFronterasEscalera = encontrarFronterasEscaleras(numCartaMano1, numCartaMano2);
			arrayInicio = arrayFronterasEscalera[0];
			arrayFinal  = arrayFronterasEscalera[1];
			
			for( int idx = 2; idx < numCartas ; ++idx ) {
				numerosEscalera[ cartas.get(idx).getNumero() ] = 1;
			}
			if( numerosEscalera[14] == 1 ) {	// Está el AS, 14-1=13
				numerosEscalera[1] = 1;
			}
			prob = probCompletarEscalera(numerosEscalera, cartasPorMostrar, arrayInicio, arrayFinal);
			
		}else {	// Nuestras 2 cartas no pueden hacer escalera de
			for( int idxCMano = 0 ; idxCMano < 2 ; ++idxCMano ) {	
				numerosEscalera = new int[15];
				numerosEscalera[ cartas.get(idxCMano).getNumero() ] = 1;	// Añadimos la carta de la mano
				arrayFronterasEscalera = encontrarFronterasEscaleras(cartas.get(idxCMano).getNumero(), -1);
				arrayInicio = arrayFronterasEscalera[0];
				arrayFinal  = arrayFronterasEscalera[1];
				for( int idx = 2; idx < numCartas ; ++idx ) {
					if( 
						cartas.get(idxCMano).mismoPaloQue(cartas.get(idx))
						) {
						numerosEscalera[ cartas.get(idx).getNumero() ] = 1;
					}
					if( numerosEscalera[14] == 1 ) {	// Está el AS, 14-1=13
						numerosEscalera[1] = 1;
					}
					prob += probCompletarEscalera(numerosEscalera, cartasPorMostrar, arrayInicio, arrayFinal);
				}
			}
		}
		
		return prob;
			
		
	}
	
	private double probCompletarEscalera( int[] numerosEscalera, int cartasPorMostrar, int arrayInicio, int arrayFinal ){
		double prob = 0.0;
		int numCartasEscalera;
		for( int idx1 = arrayInicio ; idx1 <= arrayFinal - 4 ; ++idx1 ) {
			numCartasEscalera = 0;
			for( int idx = idx1 ; idx <= idx1+4 ; ++idx) {
				if( numerosEscalera[idx] == 1 ) {
					++numCartasEscalera;
					if( numCartasEscalera >= 5 ) {
						return 1.0;
					}
				}
			}
			int cartasNecesarias = 5-numCartasEscalera;
			prob += distribucionHiperGeometrica(cartasNecesarias, cartasNecesarias*4, cartasPorMostrar);
		}
		return prob;
	}
	/*
	 * El conjunto de las siguiente funciones calcula la probabilidad de que nuestras cartas hagan FULL
	 *  Esta función tomará los posibles casos y la siguiente la probabilidad de que estos ocurran
	 */
	private double completarFullHouse( List<Carta> cartas ) {	// Falta por completar
		
		int numCartas = cartas.size();
		int cartasPorMostrar = MAX_CARTAS_VISIBLES - numCartas;
		int numParejas = 0;	// Parejas que hacen las cartas de nuestra mano
		int numTrios   = 0;	// Trios   que hacen las cartas de nuestra mano
		
		int ctt;
		
		// Esta parte de la función calculamos el número de parejas o trios que tenemos con nuestras cartas
		// En el caso de que nuestras cartas sean iguales
		if( cartas.get(IDX_CARTA_MANO_1).mismoNumeroQue(cartas.get(IDX_CARTA_MANO_2)) ) {	
			ctt = Carta.vecesQueSeSaleEsteNumero(cartas, cartas.get(IDX_CARTA_MANO_1).getNumero());
			if( ctt > 2 ) {
				++numTrios;
			}else {
				++numParejas;
			}
			
		// En el caso de que nuestas cartas no sean iguales
		}else {
			for( int idx = 0 ; idx < 2 ; ++idx ) {
				ctt = Carta.vecesQueSeSaleEsteNumero(cartas, cartas.get(idx).getNumero());
					
				if( ctt > 2 ) {
					++numTrios;
				}else if( ctt == 2 ) {
					++numParejas;
				}	
			}
		}
		
		// Ya habriamos obtenido el FULL
		if( numParejas > 0 && numTrios > 0 || numTrios > 1 ) {
			return 1.0;
			
		// Tenemos que estudiar más
		}else {
			int numParejasMesa = 0;
			int numTriosMesa = 0;
			Carta cartaAux;
			for( int idx1 = 2 ; idx1 < numCartas ; ++idx1 ) {
				cartaAux = cartas.get(idx1);
				if( !cartaAux.mismoNumeroQue(cartas.get(IDX_CARTA_MANO_1)) && !cartaAux.mismoNumeroQue(cartas.get(IDX_CARTA_MANO_2)) ) {
					ctt = 0;
					for( int idx = idx1 + 1 ; idx < numCartas ; ++idx ) {
						if( cartaAux.mismoNumeroQue(cartas.get(idx)) ) {
							++ctt;
						}
					}
					if( ctt == 2 ) {
						++numParejasMesa;
					}else if( ctt > 2 ) {
						++numTriosMesa;
					}
				}
			}
			return probCompletarFullHouse(cartasPorMostrar,  numParejas , numTrios, numParejasMesa, numTriosMesa);
		}
	}
	/*
	 * Esta función calcula la probabilidad de que salga FULL HOUSE en estos casos:
	 * 
	 */
	private double probCompletarFullHouse( int cartasPorMostrar, int numParejas, int numTrios, int numParejasMesa, int numTriosMesa ){	
		
		double prob;
		
		if( numParejas > 0 && numTriosMesa > 0 || numTrios > 0 && numParejasMesa > 0 || numTrios > 0 && numTriosMesa > 0 ) {
			prob =  1.0;
		}else {
			prob = 0.0;
			if( numTriosMesa > 0) {
				/* Necesitariamos una sola carta de la mano, para hacer el FULL*/
				prob += distribucionHiperGeometrica(1, 6, cartasPorMostrar);	
			}
			else if( numTrios > 0 ) {
				/* Necesitariamos 1 carta más de las que hay en la mesa para obtener el FULL*/
				prob += distribucionHiperGeometrica(1, (NUM_CARTAS_NUNCA_VES - cartasPorMostrar) * 3, cartasPorMostrar);
			}
			if( numParejas > 0 && numParejas + numParejasMesa > 1 ){
				/* Necesitamos una carta de alguna de las parejas para formar un trio */
				prob += distribucionHiperGeometrica(1, numParejas * 2 + numParejasMesa * 2, cartasPorMostrar);
			}
			else if( numParejas > 0 ) {
				/* Necesitamos 2 cartas de alguna de las cartas de las cartas o 3 cartas de las que no han salido todavia*/
				prob += distribucionHiperGeometrica(2, (NUM_CARTAS_NUNCA_VES - cartasPorMostrar - 2) * 3, cartasPorMostrar);
				prob += distribucionHiperGeometrica(3, 4, cartasPorMostrar);
			}else {
				prob += distribucionHiperGeometrica(5, 8, cartasPorMostrar);
			}
			
		}
		return prob;
	}
	
	private double completarPoker( List<Carta> cartas ) {
		
		double prob;
		
		int numCartas = cartas.size();
		int cartasPorMostrar = MAX_CARTAS_VISIBLES - numCartas;	
		int cttCartasPoker;
		int numCartasNecesarias;
		if( cartas.get(0).mismoNumeroQue( cartas.get(1) ) ) {
			cttCartasPoker = 2;
			
			for(  int idx = 2 ; idx < numCartas ; ++idx ) {
				if(cartas.get(0).mismoNumeroQue(cartas.get(idx))) {
					++cttCartasPoker;
				}
			}
			numCartasNecesarias = 4 - cttCartasPoker;
			prob = distribucionHiperGeometrica(numCartasNecesarias, numCartasNecesarias, cartasPorMostrar);
		}else {
			prob = 0.0;
			for(  int idxCarta = 0 ; idxCarta < 2 ; ++idxCarta ) {
				cttCartasPoker = 1;
				for( int idx = 2 ; idx < numCartas ; ++idx ) {
					if(cartas.get(idxCarta).mismoNumeroQue(cartas.get(idx))) {
						++cttCartasPoker;
					}
				}
				numCartasNecesarias = 4 - cttCartasPoker;
				prob += distribucionHiperGeometrica(numCartasNecesarias, numCartasNecesarias, cartasPorMostrar);
			}
			
		}
		
		return prob;
	}
	
	private double completarEscaleraColor( List<Carta> cartas ){
		
		double prob = 0.0;
		int numCartas = cartas.size();
		int cartasPorMostrar = MAX_CARTAS_VISIBLES - numCartas;
		int numCartaMano1 = cartas.get(IDX_CARTA_MANO_1).getNumero();
		int numCartaMano2 = cartas.get(IDX_CARTA_MANO_2).getNumero();
		int arrayInicio;
		int arrayFinal;
		
		/*
		 * El array "numerosEscaleraColor" tendrá un tamaño igual a 14
		 *  En este array podremos comprobar si tenemos un número, por ejemplo:
		 *  
		 *  Comprobamos que tenemos el AS -> numerosEscaleraColor[14] == 1
		 *  		TRUE -> Tenemos el AS
		 *  		FALSE -> No tenemos el AS
		 *  El AS al tener asociado el número 14, lo podremos comprobar en el lugar 14 y a la 1
		 *  No usaremos la posición 0
		 */
		int[] numerosEscaleraColor;	
		int[] arrayFronterasEscalera;
		
		// Nuestras 2 pueden hacer escalera color
		if( cartas.get(0).puedenHacerEscalera( cartas.get(1) ) &&  cartas.get(0).mismoPaloQue( cartas.get(1))) {	
			numerosEscaleraColor = new int[15];
			
			numerosEscaleraColor[ numCartaMano1 ] = 1;
			numerosEscaleraColor[ numCartaMano2 ] = 1;
			
			arrayFronterasEscalera = encontrarFronterasEscaleras(numCartaMano1, numCartaMano2);
			arrayInicio = arrayFronterasEscalera[0];
			arrayFinal  = arrayFronterasEscalera[1];
			
			for( int idx = 2; idx < numCartas ; ++idx ) {
				if( cartas.get(0).mismoPaloQue(cartas.get(idx)) ) {		// Que no puedan hacer escalera no importa, se filtra luego
					numerosEscaleraColor[ cartas.get(idx).getNumero() ] = 1;
				}
			}
			if( numerosEscaleraColor[14] == 1 ) {	// Está el AS, 14-1=13
				numerosEscaleraColor[1] = 1;
			}
			prob = probCompletarEscaleraColor(numerosEscaleraColor, cartasPorMostrar, arrayInicio, arrayFinal);
			
		}else {	// Nuestras 2 cartas no pueden hacer escalera de color entre ellas
			for( int idxCMano = 0 ; idxCMano < 2 ; ++idxCMano ) {	
				numerosEscaleraColor = new int[15];
				numerosEscaleraColor[ cartas.get(idxCMano).getNumero() ] = 1;	// Añadimos la carta de la mano
				arrayFronterasEscalera = encontrarFronterasEscaleras(cartas.get(idxCMano).getNumero(), -1);
				arrayInicio = arrayFronterasEscalera[0];
				arrayFinal  = arrayFronterasEscalera[1];
				for( int idx = 2; idx < numCartas ; ++idx ) {
					if( 
						cartas.get(idxCMano).mismoPaloQue(cartas.get(idx))
						) {
						numerosEscaleraColor[ cartas.get(idx).getNumero() ] = 1;
					}
					if( numerosEscaleraColor[14] == 1 ) {	// Está el AS, 14-1=13
						numerosEscaleraColor[1]  =  1;
					}
					prob += probCompletarEscaleraColor(numerosEscaleraColor, cartasPorMostrar, arrayInicio, arrayFinal);
				}
			}
		}
		
		return prob;
	}
	/*
	 * Función utilizada por las funciones: completarEscalera, completarEscaleraColor
	 * Dados 2 números o 1, devuelve un array de dimensión 2, con los límites a estudiar en el array "numerosEscaleraColor"
	 * 	arrayFronterasEscalera[0] -> Límite inferior
	 * 	arrayFronterasEscalera[1] -> Límite superior
	 */
	private int[] encontrarFronterasEscaleras( int numCarta1, int numCarta2 ) {
		int[] arrayFronterasEscalera = new int[2];
		int arrayFinal;
		int arrayInicio;
		if( numCarta1 == 14 || numCarta2 == 14 ) {	// se puede optimizar
			arrayInicio = 1;
			arrayFinal = 14;
		}else {
		
			arrayInicio = numCarta1 - 4;
			if( numCarta2 != -1 && arrayInicio < numCarta2 - 4 ) {
				arrayInicio = numCarta2 - 4;
				arrayFinal = numCarta2 + 4;
			}else {
				arrayFinal = numCarta1 + 4;
			}
			if( arrayFinal > 14 ) {
				arrayFinal = 14;
			}
			if( arrayInicio < 1 ) {
				arrayInicio = 1;
			}
		}
		arrayFronterasEscalera[0] = arrayInicio;
		arrayFronterasEscalera[1] = arrayFinal;
		
		return arrayFronterasEscalera;
	}
	
	private double probCompletarEscaleraColor( int[] numerosEscaleraColor, int cartasPorMostrar, int arrayInicio, int arrayFinal ){
		double prob = 0.0;
		int numCartasNecesarias;
		int numCartasEscalera;
		for( int idx1 = arrayInicio ; idx1 <= arrayFinal - 4 ; ++idx1 ) {
			numCartasEscalera = 0;
			for( int idx = idx1 ; idx <= idx1+4 ; ++idx) {
				if( numerosEscaleraColor[idx] == 1 ) {
					++numCartasEscalera;
					if( numCartasEscalera >= 5 ) {
						return 1.0;
					}
				}
			}
			numCartasNecesarias = 5 - numCartasEscalera;
			prob += distribucionHiperGeometrica( numCartasNecesarias, numCartasNecesarias,  cartasPorMostrar );
		}
		return prob;
	}

	/*
	 * FUNCIONA -> QUIZÁS ARREGLAR COMBINACIONES
	 */
	private double completarEscaleraReal( List<Carta> cartas ){
		
		double prob = 0.0;
		int numCartas = cartas.size();
		int cartasPorMostrar = MAX_CARTAS_VISIBLES - numCartas;
		int numCartasEscalera = 1;
		boolean cartaMano1Posible = false;
		boolean cartaMano2Posible = false;
		
		// Para hacerlo más eficiente
		int inicioBucle = 1;
		int topeBucle = 0;
		
		/* Comprobamos las cartas de nuestra mano sean aptas para la escalera real*/
		if( cartas.get(0).getNumero() >= 10 ) {
			cartaMano1Posible = true;
			inicioBucle = 0;
		}
		if( cartas.get(1).getNumero() >= 10 ) {
			cartaMano2Posible = true;
			topeBucle = 1;
		}
		
		if( cartas.get(0).mismoPaloQue( cartas.get(1)) && cartaMano1Posible && cartaMano2Posible) {
			numCartasEscalera = 2;
			topeBucle = 0;
		}
		
		for( int idxCartaMano = inicioBucle ; idxCartaMano <= topeBucle ; ++idxCartaMano ) {
			for( int idxCartaMesa = 2; idxCartaMesa < numCartas ; ++idxCartaMesa ) {
				if( 
					cartas.get(idxCartaMesa).getNumero() >= 10	&&
					cartas.get(idxCartaMano).mismoPaloQue(cartas.get(idxCartaMesa))			
					) {
					++numCartasEscalera;	
				}
			}
			int cartasNecesarias = 5 - numCartasEscalera;
			prob += distribucionHiperGeometrica(cartasNecesarias,cartasNecesarias, cartasPorMostrar);	/* Las cartas necesarias son las mismas, a las válidas */
			numCartasEscalera = 1;
		}
			
		return prob;
	}
	
	/*
	 * Mediante la distribución HiperGeométrica
	 * Devolverá la probabilidad de que la mano X  se complete
	 */
	private double distribucionHiperGeometrica( int cartasNecesarias, int cartasValidasRestantes, int cartasPorMostrar) {
		double prob;
		if( cartasNecesarias <= 0 ) {
			prob = 1.0;
		}else if( cartasNecesarias > cartasPorMostrar ) {
			prob = 0.0;
		}else{
			int combinacionesPosibles = C( NUM_CARTAS_NUNCA_VES + cartasPorMostrar , cartasPorMostrar );
			int combinacionesFormasSalirCartasNecesarias = C( cartasValidasRestantes , cartasNecesarias );
			int combinacionesDemasCartas = C( NUM_CARTAS_NUNCA_VES + (cartasPorMostrar-cartasNecesarias), (cartasPorMostrar-cartasNecesarias));
			
			prob = (double) (combinacionesFormasSalirCartasNecesarias * combinacionesDemasCartas) / combinacionesPosibles;
		}
		
		return prob;
	}
	
	
	private double distribucionBinomial(int cartasNecesarias, int cartasPorMostrar) {	// No la uso
		
		int cartasQueNoHanSalido = 45 + cartasPorMostrar;			//N = cartasQueNoHanSalido ; K = Cartas Necesarias
		
		long combinacionesNK = C(cartasQueNoHanSalido, cartasNecesarias);
		double p = cartasNecesarias / cartasQueNoHanSalido;
		
		return (double) combinacionesNK * Math.pow(p, cartasNecesarias) * Math.pow(1.0-p, cartasQueNoHanSalido-cartasNecesarias);
	}
	/*
	 * ##############################################################################################################
	 * 								CALCULAR PROBABILIDAD DE LOS CONTRINCANTES
	 * ##############################################################################################################
	 */
}

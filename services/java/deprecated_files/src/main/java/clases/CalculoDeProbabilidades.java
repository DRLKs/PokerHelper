package main.java.clases;

/*
 * MADE BY DRLK
 */
import java.util.HashMap;
import java.util.List;
import java.util.Map;

public class CalculoDeProbabilidades {
	/*
	 * Todas las probabilidades son sobre 1
	 */
	private double probPareja;
	private double probTrio;
	private double probEscalera;
	private double probColor;
	private double probFullHouse;
	private double probPoker;
	private double probEscaleraColor;
	private double probEscaleraReal;
	
	private double probParejaCont;
	private double probTrioCont;
	private double probEscaleraCont;
	private double probColorCont;
	private double probFullHouseCont;
	private double probPokerCont;
	private double probEscaleraColorCont;
	private double probEscaleraRealCont;
	
	/**
	 * Esta variable definirá la decisión que debe tomar el jugador<br> 
	 */
	private Decision decision;
	
	/**
	 * El número máximo de cartas que podemos ver al mismo tiempo son 7
	 */
	private final int MAX_CARTAS_VISIBLES = 7;
	/**
	 * El número de cartas que tienen el mismo número es 4
	 */
	private final int MAX_CARTAS_NUMERO = 4;
	private final int IDX_CARTA_MANO_1 = 0;
	private final int IDX_CARTA_MANO_2 = 1;
	/**
	 * 45
	 */
	private final int NUM_CARTAS_NUNCA_VES = 45;

	/**
	 * Función que recalcula todas las probabilidades
	 * 
	 * @param cartas Lista que contiene las cartas conocidas
	 * @param numContrincantes Número de contrincantes activos
	 * @param ciegaPequenya Valor de la ciega pequenya de la mano
	 * @param apuestaAcumulada Valor de la apuesta acumulada en la mano
	 */
	public void reiniciarDatos( List<Carta> cartas, int numContrincantes, int ciegaPequenya, int apuestaAcumulada) {
		probPareja = completarPareja(cartas);
		probTrio = completarTrio(cartas);
		probEscalera = completarEscalera(cartas);
		probColor = completarColor(cartas);
		probFullHouse = completarFullHouse(cartas);
		probPoker = completarPoker(cartas);
		probEscaleraColor = completarEscaleraColor(cartas);
		probEscaleraReal = completarEscaleraReal(cartas);
		
		completarParejaTrioPokerCont(cartas, numContrincantes);
		probEscaleraCont = completarEscaleraCont(cartas, numContrincantes);
		probColorCont = completarColorCont( cartas, numContrincantes );
		probFullHouseCont = completarFullHouseCont(cartas, numContrincantes);
		probEscaleraColorCont = completarEscaleraColorCont(cartas, numContrincantes);
		probEscaleraRealCont = completarEscaleraRealCont(cartas, numContrincantes);
		
		decision = calcularDecision(ciegaPequenya, apuestaAcumulada);

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

	/*
	 * ##############################################################################################################
	 * 												GETTERS 
	 * ##############################################################################################################
	 */

	public double getProbPareja() {
		return probPareja;
	}
	
	public double getProbTrio() {
		return probTrio;
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
	

	public double getProbParejaCont() {
		return probParejaCont;
	}
	
	public double getProbTrioCont() {
		return probTrioCont;
	}
	
	public double getProbColorCont() {
		return probColorCont;
	}
	
	public double getProbEscaleraCont() {
		return probEscaleraCont;
	}

	public double getProbFullHouseCont() {
		return probFullHouseCont;
	}


	public double getProbPokerCont() {
		return probPokerCont;
	}
	
	public double getProbEscaleraColorCont() {
		return probEscaleraColorCont;
	}

	public double getProbEscaleraRealCont() {
		return probEscaleraRealCont;
	}

	public Decision getDecision() {
		return decision;
	}

	public void setDecision(Decision decision) {
		this.decision = decision;
	}
	
	
	/**
	 * Función que devuelve las combinaciones entre 2 números
	 * 
	 * @param n Número de elementos en el conjunto
	 * @param k Número de combinaciones
	 * @return Número de combinaciones posibles
	 */
	public int C( int n ,int k) { // COMBINACIONES
		
		if (k > n || k < 0) return -1; // No existe combinatoria si k > n
        if (k == 0 || k == n ) return 1;
        /* La combinación es simétrica, C(n, k) = C(n, n-k) */
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
	 * ##############################################################################################################
	 * 								FUNCIONES PARA CALCULAR NUESTRAS PROBABILIDADES
	 * ##############################################################################################################
	 */
	
	/** 
	 * Función que devuelve la probabilidad de obtener una pareja con las cartas que tenemos
	 * 
	 * @param cartas Lista que contiene las cartas conocidas
	 * @return Probabilidad de obtener una pareja con las cartas de nuestra mano (double)
	 * */
	private double completarPareja( List<Carta> cartas ) {
		
		double prob;

		int numCartas = cartas.size();
		int cartasPorMostrar = MAX_CARTAS_VISIBLES - numCartas;
		int cartasTotales = NUM_CARTAS_NUNCA_VES + cartasPorMostrar;

		if( cartas.get(IDX_CARTA_MANO_1).mismoNumeroQue(cartas.get(IDX_CARTA_MANO_2)) ) {	/* Nuestras cartas hacen pareja */
			prob = 1.0;
		}else {					/* Nuestras cartas no hacen pareja */
			int num;
			prob = 0.0;
			
			for( int idxCartaMano = 0 ; idxCartaMano < 2 ; ++idxCartaMano ) {
				num = cartas.get(idxCartaMano).getNumero(); 	/* Número de nuestra carta */
				
				for( int idx = 2 ; idx < numCartas ; idx++ ) {
					if( cartas.get(idx).getNumero() == num ) {	/* Encontramos carta en la mesa con el mismo número */
						prob = 1.0;
					}
				}
			}
			
			if( prob <= 0.0 ) {	/* En el caso de que la probabilidad siga siendo 0.0, no hemos encontrado ninguna pareja */
				prob += distribucionHiperGeometrica(1, MAX_CARTAS_NUMERO - 1, cartasTotales, cartasPorMostrar);
			}
			
		}
		return prob;
	}
	
	/** 
	 * Función que devuelve la probabilidad de obtener un trio con las cartas que tenemos
	 * 
	 * @param cartas Lista que contiene las cartas conocidas
	 * @return Probabilidad de obtener un trio con las cartas de nuestra mano (double)
	 * */
	private double completarTrio( List<Carta> cartas ) {
		
		double prob;
		int numCartasTrio;

		int numCartas = cartas.size();
		int cartasPorMostrar = MAX_CARTAS_VISIBLES - numCartas;
		int cartasTotales = NUM_CARTAS_NUNCA_VES + cartasPorMostrar;

		if( cartas.get(IDX_CARTA_MANO_1).mismoNumeroQue(cartas.get(IDX_CARTA_MANO_2)) ) {	/* Nuestras cartas hacen pareja */
			numCartasTrio = 2;
			int num = cartas.get(IDX_CARTA_MANO_1).getNumero(); /* Número de nuestras cartas */
			for( int idx = 2 ; idx < numCartas ; ++idx ) {
				
				if( cartas.get(idx).getNumero() == num ) {	/* Ya tenemos un trio */
					numCartasTrio = 3;
					break;
				}
			}
			prob = distribucionHiperGeometrica(3 - numCartasTrio, MAX_CARTAS_NUMERO - numCartasTrio, cartasTotales, cartasPorMostrar);
			
		}else {					/* Nuestras cartas no hacen pareja */
			int num;
			prob = 0.0;
			
			for( int idxCartaMano = 0 ; idxCartaMano < 2 ; ++idxCartaMano ) {
				numCartasTrio = 1;
				num = cartas.get(idxCartaMano).getNumero(); 	/* Número de nuestra carta */
				
				for( int idx = 2 ; idx < numCartas ; idx++ ) {
					if( cartas.get(idx).getNumero() == num ) {	/* Encontramos carta en la mesa con el mismo número */
						++numCartasTrio;
					}
				}
				
				prob += distribucionHiperGeometrica(3 - numCartasTrio, MAX_CARTAS_NUMERO - numCartasTrio, cartasTotales, cartasPorMostrar);

			}

		}
		return prob;
	}
	
	/** 
	 * Función que devuelve la probabilidad de obtener color con las cartas que tenemos
	 * 
	 * @param cartas Lista que contiene las cartas conocidas
	 * @return Probabilidad de obtener color con las cartas de nuestra mano (double)
	 * */
	private double completarColor( List<Carta> cartas ){
		
		double prob;
		int numCartasColor = 1;
		int numCartas = cartas.size();
		int cartasPorMostrar = MAX_CARTAS_VISIBLES - numCartas;
		int cartasTotales = NUM_CARTAS_NUNCA_VES + cartasPorMostrar;
		int numCartasNecesarias;
		if( cartas.get(0).mismoPaloQue( cartas.get(1) ) ) {
			++numCartasColor;
			
			for( int i = 2; i < numCartas ; ++i ) {
				if( cartas.get(i).mismoPaloQue(cartas.get(0)) ) {
					++numCartasColor;
				}
			}
			numCartasNecesarias = 5-numCartasColor;
			prob = distribucionHiperGeometrica(numCartasNecesarias, 13-numCartasColor, cartasTotales, cartasPorMostrar);
		}else {
			
			prob = 0.0;
			for( int idxCMano = 0 ; idxCMano < 2 ; ++idxCMano ) {	
					for( int i = 2; i < numCartas ; ++i ) {
						if( cartas.get(i).mismoPaloQue(cartas.get(idxCMano)) ) {
							++numCartasColor;
					}
				}
				numCartasNecesarias = 5 - numCartasColor;
				prob += distribucionHiperGeometrica(numCartasNecesarias, 13-numCartasColor, cartasTotales, cartasPorMostrar);
				numCartasColor = 1;
			}
		}
		return prob;
	}
	
	/**
	 * Función que devuelve la probabilidad de obtener escalera con las cartas que tenemos
	 * 
	 * @param cartas Lista que contiene las cartas conocidas
	 * @return Probabilidad de obtener escalera con las cartas de nuestra mano (double)
	 */
	private double completarEscalera( List<Carta> cartas ){
			
		double prob;
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
		
		/* Nuestras 2 cartas pueden hacer escalera */
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
		/* Nuestras 2 cartas no pueden hacer escalera */
		}else {	
			prob = 0.0;
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
					if( numerosEscalera[14] == 1 ) {
						numerosEscalera[1] = 1;
					}
					prob += probCompletarEscalera(numerosEscalera, cartasPorMostrar, arrayInicio, arrayFinal);
				}
			}
		}
		return prob;	
		
	}
	
	/**
	 * Función auxiliar que ayuda a obtener la probabilidad de obtener una escalera con las cartas que tenemos
	 *
	 * @param numerosEscalera Array con los números de las cartas que tenemos
	 * @param cartasPorMostrar Número de cartas que faltan por mostrar
	 * @param arrayInicio Índice del array por el cual comenzaremos a estudiar la escalera (Frontera Por Debajo)
	 * @param arrayFinal Índice del array por el cual terminaremos de estudiar la escalera (Frontera Por Encima)
	 * @return Probabilidad de obtener escalera con una carta específica de nuestra mano (double)
	 */
	private double probCompletarEscalera( int[] numerosEscalera, int cartasPorMostrar, int arrayInicio, int arrayFinal ){
		double prob = 0.0;
		int numCartasEscalera;
		int cartasTotales = NUM_CARTAS_NUNCA_VES + cartasPorMostrar;
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
			prob += distribucionHiperGeometrica(cartasNecesarias, cartasNecesarias*4,cartasTotales, cartasPorMostrar);
		}
		return prob;
	}

	/**
	 * Función que devuelve la probabilidad de obtener un Full con las cartas que tenemos
	 *
	 * @param cartas Lista que contiene las cartas conocidas
	 * @return Probabilidad de obtener un Full con las cartas de nuestra mano (double)
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

	/**
	 * Función auxiliar que ayuda a obtener la probabilidad de tener un Full con las cartas que tenemos
	 *
	 * @return Probabilidad de obtener una escalera de color con las cartas de nuestra mano (double)
	 */
	private double probCompletarFullHouse( int cartasPorMostrar, int numParejas, int numTrios, int numParejasMesa, int numTriosMesa ){	
		
		double prob;
		
		/* Ya tendriamos el poker */
		if( numParejas > 0 && numTriosMesa > 0 || numTrios > 0 && numParejasMesa > 0 || numTrios > 0 && numTriosMesa > 0 ) {
			prob =  1.0;
		}else {
		/* Debemos seguir estudiandolo */	
			prob = 0.0;
			int cartasTotales = NUM_CARTAS_NUNCA_VES + cartasPorMostrar;
			if( numTriosMesa > 0) {
				/* Necesitariamos una sola carta de la mano, para hacer el FULL*/
				prob += distribucionHiperGeometrica(1, (MAX_CARTAS_NUMERO - 1) * 2,cartasTotales, cartasPorMostrar);	
			}
			else if( numTrios > 0 ) {
				/* Necesitariamos 1 carta más de las que hay en la mesa para obtener el FULL*/
				prob += distribucionHiperGeometrica(1, (MAX_CARTAS_VISIBLES - cartasPorMostrar) * 3,cartasTotales, cartasPorMostrar);
				/* Necesitamos 2 cartas más de las que no han salido en la mesa */
				prob += distribucionHiperGeometrica(2, MAX_CARTAS_NUMERO ,cartasTotales, cartasPorMostrar);
			}
			
			if( numParejas > 0 && numParejas + numParejasMesa > 1 ){
				/* Necesitamos una carta de alguna de las parejas para formar un trio */
				prob += distribucionHiperGeometrica(1, numParejas * 2 + numParejasMesa * 2,cartasTotales, cartasPorMostrar);
				/* Necesitamos 3 cartas más de algun número que no ha salido todavía */
				prob += distribucionHiperGeometrica(3, MAX_CARTAS_NUMERO,cartasTotales, cartasPorMostrar);
			}
			else if( numParejas > 0 ) {
				/* Necesitamos 2 cartas de alguna de las cartas de las cartas */
				prob += distribucionHiperGeometrica(2, (MAX_CARTAS_VISIBLES - cartasPorMostrar - 2) * 3,cartasTotales, cartasPorMostrar);
				/* Necesitamos 3 cartas de las que no han salido todavia */
				prob += distribucionHiperGeometrica(3, MAX_CARTAS_NUMERO,cartasTotales, cartasPorMostrar);
			}else {
				/* Obtener pareja y trio de las cartas de nuestra mano */
				prob += distribucionHiperGeometrica(3, 6,cartasTotales, cartasPorMostrar);
			}
			
		}
		return prob;
	}
	
	/**
	 * Función que devuelve la probabilidad de obtener Poker con las cartas que tenemos
	 *
	 * @param cartas Lista que contiene las cartas conocidas
	 * @return Probabilidad de obtener Poker con las cartas de nuestra mano (double)
	 */
	private double completarPoker( List<Carta> cartas ) {
		
		double prob;
		
		int numCartas = cartas.size();
		int cartasPorMostrar = MAX_CARTAS_VISIBLES - numCartas;
		int cartasTotales = NUM_CARTAS_NUNCA_VES + cartasPorMostrar;
		int cttCartasPoker;
		int numCartasNecesarias;
		if( cartas.get(0).mismoNumeroQue( cartas.get(1) ) ) {
			cttCartasPoker = 2;
			
			for(  int idx = 2 ; idx < numCartas ; ++idx ) {
				if(cartas.get(0).mismoNumeroQue(cartas.get(idx))) {
					++cttCartasPoker;
				}
			}
			numCartasNecesarias = MAX_CARTAS_NUMERO - cttCartasPoker;
			prob = distribucionHiperGeometrica(numCartasNecesarias, numCartasNecesarias, cartasTotales, cartasPorMostrar);
		}else {
			prob = 0.0;
			for(  int idxCarta = 0 ; idxCarta < 2 ; ++idxCarta ) {
				cttCartasPoker = 1;
				for( int idx = 2 ; idx < numCartas ; ++idx ) {
					if(cartas.get(idxCarta).mismoNumeroQue(cartas.get(idx))) {
						++cttCartasPoker;
					}
				}
				numCartasNecesarias = MAX_CARTAS_NUMERO - cttCartasPoker;
				prob += distribucionHiperGeometrica(numCartasNecesarias, numCartasNecesarias,cartasTotales, cartasPorMostrar);
			}
			
		}
		
		return prob;
	}
	
	/**
	 * Función que devuelve la probabilidad de obtener escalera de color con las cartas que tenemos
	 *
	 * @param cartas Lista que contiene las cartas conocidas
	 * @return Probabilidad de obtener una escalera de color con las cartas de nuestra mano (double)
	 */
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
	/**
	 * Función utilizada por las funciones: completarEscalera, completarEscaleraColor. Lo que hace esta función es determinar 
	 * los valores que debemos analizar para la obtención de la probabilidad de conseguir escaleras
	 *  
	 * @param numCarta1 Número de la carta
	 * @param numCarta2 Número de la segunda carta o -1 para indicar que solo tenemos una
	 * @return Un array de dimensión 2, donde en la primera posición nos encontramos el límite inferior y en la segunda el límite superior
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
	
	/**
	 * Función auxiliar que ayuda a obtener la probabilidad de obtener una escalera de color con las cartas que tenemos
	 *
	 * @param numerosEscaleraColor Array con los números de las cartas que tenemos
	 * @param cartasPorMostrar Número de cartas que faltan por mostrar
	 * @param arrayInicio Índice del array por el cual comenzaremos a estudiar la escalera (Frontera Por Debajo)
	 * @param arrayFinal Índice del array por el cual terminaremos de estudiar la escalera (Frontera Por Encima)
	 * @return Probabilidad de obtener escalera de color con una carta específica de nuestra mano (double)
	 */
	private double probCompletarEscaleraColor( int[] numerosEscaleraColor, int cartasPorMostrar, int arrayInicio, int arrayFinal ){
		double prob = 0.0;
		int numCartasNecesarias;
		int numCartasEscalera;
		int cartasTotales = NUM_CARTAS_NUNCA_VES + cartasPorMostrar;
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
			prob += distribucionHiperGeometrica( numCartasNecesarias, numCartasNecesarias, cartasTotales,  cartasPorMostrar );
		}
		return prob;
	}

	/**
	 * Función que devuelve la probabilidad de obtener escalera real con las cartas que tenemos
	 *
	 * @param cartas Lista que contiene las cartas conocidas
	 * @return Probabilidad de obtener escalera real con las cartas de nuestra mano (double)
	 */
	private double completarEscaleraReal( List<Carta> cartas ){
		
		double prob = 0.0;
		int numCartas = cartas.size();
		int cartasPorMostrar = MAX_CARTAS_VISIBLES - numCartas;
		int cartasTotales = NUM_CARTAS_NUNCA_VES + cartasPorMostrar;
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
			prob += distribucionHiperGeometrica(cartasNecesarias,cartasNecesarias,cartasTotales, cartasPorMostrar);	/* Las cartas necesarias son las mismas, a las válidas */
			numCartasEscalera = 1;
		}
			
		return prob;
	}
	
	/*
	 * ##############################################################################################################
	 * 								CALCULAR PROBABILIDAD DE LOS CONTRINCANTES
	 * ##############################################################################################################
	 */
	
	/** 
	 * Función que calcula la probabilidad de que uno de nuestro contrincantes obtengan una pareja o un trio o un poker con las cartas que conocemos
	 * 
	 * @param cartas Lista que contiene las cartas conocidas
	 * @param numContrincantes número de contrincantes activos
	 * */
	private void completarParejaTrioPokerCont( List<Carta> cartas, int numContrincantes ) {
		
		int numCartas = cartas.size();
		/* Si solo tenemos nuestrar cartas no calculamos nada de los contrincantes*/
		if( numCartas < 3 ) {
			this.probParejaCont = -1;
			this.probTrioCont = -1;
			this.probPokerCont = -1;

		}
		else 
		{
			this.probParejaCont = 0;
			this.probTrioCont = 0;
			this.probPokerCont = 0;
			
			int cartasPorMostrar = MAX_CARTAS_VISIBLES - numCartas;
			int cartasTotales = NUM_CARTAS_NUNCA_VES + cartasPorMostrar;
			int numCartasNecesarias;
			int numCartasValidasRestantes = 4;	/* Solo hay 4 cartas por número */
			int numCartasQueYoTengo;
	
			Carta cartaAux;
			int cttCartasNumero;
			
			/* Rellenamos el Map */
			Map< Integer , Integer> contarCartas = new HashMap<>();
			for( int idx = 2 ; idx < numCartas ; ++idx ) {
				cartaAux = cartas.get(idx);
				cttCartasNumero = contarCartas.getOrDefault( cartaAux.getNumero(), 0) + 1;
				contarCartas.put(cartaAux.getNumero(),cttCartasNumero);
			}
			
			for( Map.Entry<Integer , Integer> tupla : contarCartas.entrySet() ) {
				numCartasQueYoTengo = 0;
				/* Contamos las cartas de este tipo que tenemos */
				for( int idx = 0 ; idx < 2 ; ++idx ) {
					if( cartas.get(idx).getNumero() == tupla.getKey() ) {
						++numCartasQueYoTengo;
					}
				}
			
				/* Las cartas que no podemos ver y ayudarían a completar la mano */
				numCartasValidasRestantes = 4 - tupla.getValue() - numCartasQueYoTengo;
				
				/* Las cartas que necesita el contrincante para conseguir una pareja */
				numCartasNecesarias  = 2 - tupla.getValue();
				this.probParejaCont += distribucionHiperGeometrica(numCartasNecesarias, numCartasValidasRestantes,cartasTotales, cartasPorMostrar);
				
				
				/* Las cartas que no podemos ver y ayudarían a completar el trio */
				numCartasNecesarias  = 3 - tupla.getValue();
				this.probTrioCont += distribucionHiperGeometrica(numCartasNecesarias, numCartasValidasRestantes,cartasTotales, cartasPorMostrar);
				
				
				/* Las cartas que no podemos ver y ayudarían a completar el poker */
				numCartasNecesarias  = 4 - tupla.getValue();
				this.probPokerCont += distribucionHiperGeometrica(numCartasNecesarias, numCartasValidasRestantes,cartasTotales, cartasPorMostrar);
				
			}
		}
	}
	
	/**
	 * Función que devuelve la probabilidad de que uno de nuestro contrincantes obtenegan color con las cartas que conocemos
	 * 
	 * @param cartas Lista que contiene las cartas conocidas
	 * @param numContrincantes Número de contrincantes activos
	 * @return Probabilidad de que un contrincante obtenga color con las cartas que conocemos (double)
	 */
	private double completarColorCont(List<Carta> cartas, int numContrincantes) {
		
		int numCartas = cartas.size();
		/* Si solo tenemos nuestrar cartas no calculamos nada de los contrincantes*/
		if( numCartas < 3 ) {
			return -1;
		}
		double prob = 0.0;
		int cartasPorMostrar = MAX_CARTAS_VISIBLES - numCartas;
		int cartasTotales = NUM_CARTAS_NUNCA_VES + cartasPorMostrar;
		int numCartasNecesarias;
		int numCartasValidasRestantes = 13;
		int numCartasQueYoTengo;
		
		int cttCartasColor;
		Carta cartaAux;
		Map< Character , Integer> contarCartas = new HashMap<>();
		
		for( int idx = 2 ; idx < numCartas ; ++idx ) {
			cartaAux = cartas.get(idx);
            cttCartasColor = contarCartas.getOrDefault( cartaAux.getPalo(), 0) + 1;
			contarCartas.put(cartaAux.getPalo(),cttCartasColor);
		}
		
		for( Map.Entry<Character , Integer> tupla : contarCartas.entrySet() ) {
			
			/* Cuento las cartas que tengo de este color */
			numCartasQueYoTengo = 0;
			for( int idx = 0 ; idx < 2 ; ++idx ) {
				if( cartas.get(idx).getPalo() == tupla.getKey() ) {
					++numCartasQueYoTengo;
				}
			}
			/* Las cartas que necesita el contrincante para conseguir color */
			numCartasNecesarias  = 5 - tupla.getValue();
			
			/* Las cartas que no podemos ver y ayudarían a completar el color */
			numCartasValidasRestantes -= tupla.getValue() + numCartasQueYoTengo;
			
			for( int numCartasContrTendria = 0 ; numCartasContrTendria < numCartasNecesarias ; ++numCartasContrTendria ) {
				prob += distribucionHiperGeometrica(numCartasNecesarias, numCartasValidasRestantes,cartasTotales, cartasPorMostrar) *  distribucionHiperGeometrica(numCartasContrTendria, numCartasValidasRestantes,numContrincantes*2, cartasPorMostrar);
				/* Esto esta raro, no me acuerdo lo que hace aquí */
			}
		}
		
		return prob;
	}
	
	private double completarEscaleraCont(List<Carta> cartas, int numContrincantes) {
		
		int numCartas = cartas.size();
		/* Si solo tenemos nuestrar cartas no calculamos nada de los contrincantes*/
		if( numCartas < 3 ) {
			return -1;
		}
		double prob = 0.0;
		int cartasPorMostrar = MAX_CARTAS_VISIBLES - numCartas;
		int cartasTotales = NUM_CARTAS_NUNCA_VES + cartasPorMostrar;
		//int numCartasNecesarias;
		//int numCartasValidas = 13;
		//int numCartasQueYoTengo;
		
		
		int[] numerosEscaleraCont = new int[15];
		
		for( int idx = 2; idx < numCartas ; ++idx ) {
			numerosEscaleraCont[ cartas.get(idx).getNumero() ] = 1;
		}
		if( numerosEscaleraCont[14] == 1 ) {	// Está el AS, 14-1=13
			numerosEscaleraCont[1] = 1;
		}
		
		/* Calculamos los límites de la baraja que debemos estudiar */
		
		return prob;
	}
	
	private double completarFullHouseCont(List<Carta> cartas, int numContrincantes) {
		return 0.0;
	}
	
	private double completarEscaleraColorCont(List<Carta> cartas, int numContrincantes) {
		return 0.0;
	}
	
	private double completarEscaleraRealCont(List<Carta> cartas, int numContrincantes) {
		return 0.0;
	}
	
	/*
	 * ##############################################################################################################
	 * 												TOMA DE DECISIONES
	 * ##############################################################################################################
	 */

	/**
	 * 
	 * @param ciegaPequenya Valor de la ciega pequenya de la mano
	 * @param apuestaAcumulada Valor de la apuesta acumulada en la mano
	 * 
	 * @return Clase que define la decisión que debe tomar el jugador
	 */
	private Decision calcularDecision(int ciegaPequenya, int apuestaAcumulada) {
		
		Decision decision = new Decision();
		
		if( probEscaleraReal == 1.0 ) {				/* JUGADOR TIENE ESCALERA REAL (Mejor mano posible, no la puede tener nadie a la vez) */
			decision.setDecision( Decision.ALL_IN );
		}else if( probEscaleraColor == 1.0 && probEscaleraColorCont != 1.0) {
			decision.setDecision( Decision.ALL_IN );
		}else if( probPoker == 1.0 ) {
			decision.setDecision( Decision.ALL_IN );
		}
		
		
		return decision;
	}
	
	
	/*
	 * ##############################################################################################################
	 * 									FUNCIONES PARA CALCULAR PROBABILIDAD 
	 * ##############################################################################################################
	 */
	
	/**
	 * Esta función usa la distribución hipergeométrica para obtener la probabilidad de obtener una determinada mano
	 * 
	 * @param cartasNecesarias El número de cartas que necesitamos para completar nuestra mano
	 * @param cartasValidasRestantes El número de cartas de ese tipo que nos sirven para completar la mano
	 * @param cartasTotales El número de cartas totales que no han salido todavía
	 * @param cartasPorMostrar El número de cartas que faltan por desvelar 
	 * 
	 * @return La probabilidad de que la mano se complete
	 */
	private double distribucionHiperGeometrica( int cartasNecesarias, int cartasValidasRestantes, int cartasTotales, int cartasPorMostrar) {
		double prob;
		if( cartasNecesarias <= 0 ) {
			prob = 1.0;
		}else if( cartasNecesarias > cartasPorMostrar ) {
			prob = 0.0;
		}else{
			int combinacionesPosibles = C( cartasTotales , cartasPorMostrar );
			int combinacionesFormasSalirCartasNecesarias = C( cartasValidasRestantes , cartasNecesarias );
			int combinacionesDemasCartas = C( cartasTotales - cartasValidasRestantes, cartasPorMostrar - cartasNecesarias);
			
			prob = (double) (combinacionesFormasSalirCartasNecesarias * combinacionesDemasCartas) / combinacionesPosibles;
		}
		
		return prob;
	}
	
	/**
	 * Esta función usa la distribución binomial para obtener la probabilidad de obtener una determinada mano
	 * 
	 * @param cartasNecesarias El número de cartas que necesitamos para completar nuestra mano
	 * @param cartasPorMostrar El número de cartas que faltan por desvelar 
	 * 
	 * @deprecated
	 * 
	 * @return La probabilidad de que la mano se complete
	 */
	private double distribucionBinomial(int cartasNecesarias, int cartasPorMostrar) {
		
		int cartasQueNoHanSalido = 45 + cartasPorMostrar;			//N = cartasQueNoHanSalido ; K = Cartas Necesarias
		
		long combinacionesNK = C(cartasQueNoHanSalido, cartasNecesarias);
		double p = cartasNecesarias / cartasQueNoHanSalido;
		
		return (double) combinacionesNK * Math.pow(p, cartasNecesarias) * Math.pow(1.0-p, cartasQueNoHanSalido-cartasNecesarias);
	}
	
	
	/**
	 * Obtenemos la probabilidad de obtener un determinado número de cartas, en un conjunto donde tenemos un numero de cartas válidas y del cual solo podemos tomar un número límitado de estas
	 *
	 * @deprecated
	 *
	 * @param numCartasValidas número de cartas del conjunto que son válidas para el individuo
	 * @param numCartasPosibles número de cartas diferentes que podemos tomar
	 * @param iteraciones número de cartas que posee el conjunto con el que trabajamos
	 * @param numCartasNecesarias número de cartas del conjunto de cartas válidas que necesitamos
	 * 
	 */
	private double probabilidadComplementaria( int numCartasValidas, int numCartasPosibles, int iteraciones, int numCartasNecesarias) {
		double prob = 1.0;
		
		int numCartasPosiblesAux;
		int numCartasValidasAux;
		
		if( numCartasNecesarias <= 2 && numCartasNecesarias > 0) {
			
			for( int co = 0 ; co < numCartasNecesarias ; co++ ) {	// co = cartas que obtienes

				for( int io = 0 ; io < numCartasNecesarias ; io++ ) {	// io = iteracion donde obtiene la carta

					numCartasValidasAux = numCartasValidas;
					numCartasPosiblesAux = numCartasPosibles;
					
					for( int i = 0 ; i < iteraciones ; ++i ) {
						
						if( i == io && co > 0  ) {
							prob *= (double) numCartasValidasAux / numCartasPosiblesAux;
							--numCartasValidasAux;
						}else {
							prob *=  1.0 -  ((double) numCartasValidasAux / numCartasPosiblesAux);
							--numCartasPosiblesAux;
						}
						
					}
				}
			}
			
			prob = 1.0 - prob;
			
		}else if( numCartasNecesarias > 0) {	// Es mejor cambiar el método para estos casos para no elevar mucho la complejidad del algoritmo
			
			
			
		}
		
		return prob;
	}

	
}

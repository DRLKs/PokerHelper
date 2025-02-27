package main.java.gui;

import java.util.List;
import java.util.concurrent.ExecutionException;

import javax.swing.SwingWorker;

import main.java.clases.*;

public class Worker extends SwingWorker<Integer , Integer>{
	
	private List<Carta> cartas;
	private int numContrincantes;
	private int ciegaPequenya;
	private int apuestaAcumulada;
	private CalculoDeProbabilidades calc;
	private Controlador controlador;
	
	public Worker( int numContrincantes, List<Carta> cartas, Controlador controlador, int ciegaPequenya, int apuestaAcumulada) {
		this.cartas = cartas;
		this.controlador = controlador;
		this.numContrincantes = numContrincantes;
		this.ciegaPequenya = ciegaPequenya;
		this.apuestaAcumulada = apuestaAcumulada;
		this.calc = new CalculoDeProbabilidades();
	}
	
	/**
	 * Función que llama al SwingWorker
	 * Hilo que calcula la probabilidad en BackGround
	 */
	@Override
	protected Integer doInBackground() throws Exception {
		/*
		if( cartas.size() == 2 ) {
			return calcularProbabilidadSalida();
		}else if( cartas.size() == 5 ) {
			return 0;
		}else if(  cartas.size() == 4 ) {
			return 0;
		}else {
			return calcularProbabilidad();
		}
		*/
		return calcularProbabilidad();
		
	}
	/*
	 * Cuando termina doInBackground()
	 * get() = valor que retorna doInBackground()
	 */
	@Override
	public void done() {
		
		try {
			int decision = get();
			
			switch (decision) {
			case 0: 
				break;
			case 1:
				break;
			default:
			}
			

		} catch (InterruptedException | ExecutionException e) {
			e.printStackTrace();
		} catch( Exception e) {
			e.printStackTrace();
		}
	}
	
	/**
	 * Función llamada en segundo plano, esta hace todos los calculos probabilísticos
	 * 
	 * <b> POR TERMINAR </b>
	 * 
	 * @return Devuelve un entero que identifica a una decisión que debe tomar el jugador
	 */
	private int calcularProbabilidad() {
		
		calc.reiniciarDatos(cartas , numContrincantes, ciegaPequenya, apuestaAcumulada);
		
		sacarDatosObtenidos();
		int prob =(int)(100*calc.getProbEscaleraColor()) ;
		System.out.println(".............."+ prob);
		return calc.getDecision().getCodigoDecision();
	}
	
	private void sacarDatosObtenidos() {
		System.out.println("-------------------------------------");
		System.out.println("Probabilidad pareja: " + calc.getProbPareja());
		
		System.out.println("Probabilidad trio: " + calc.getProbTrio());
		System.out.println("Probabilidad escalera: " + calc.getProbEscalera());
		System.out.println("Probabilidad color: " + calc.getProbColor());
		System.out.println("Probabilidad FullHouse: " + calc.getProbFullHouse());
		System.out.println("Probabilidad Poker: " + calc.getProbPoker());
		System.out.println("Probabilidad escalera de color : " + calc.getProbEscaleraColor());
		System.out.println("Probabilidad escalera real: " + calc.getProbEscaleraReal());

		System.out.println("Probabilidad pareja contr: " + calc.getProbParejaCont());
		System.out.println("Probabilidad trio contr: " + calc.getProbTrioCont());
		System.out.println("Probabilidad escalera contr: " + calc.getProbEscaleraCont());
		System.out.println("Probabilidad color contr contr: " + calc.getProbColorCont());
		System.out.println("Probabilidad FullHouse contr: " + calc.getProbFullHouseCont());
		System.out.println("Probabilidad Poker contr: " + calc.getProbPokerCont());
		System.out.println("Probabilidad escalera de color contr: " + calc.getProbEscaleraColorCont());
		System.out.println("Probabilidad escalera real contr: " + calc.getProbEscaleraRealCont());
		
		System.out.println("-------------------------------------");
		
		
		controlador.setProbabilidad(0, (int) (calc.getProbPareja() 		  * 100) );
		controlador.setProbabilidad(1, (int) (calc.getProbTrio() 		  * 100) );
		controlador.setProbabilidad(2, (int) (calc.getProbEscalera() 	  * 100) );
		controlador.setProbabilidad(3, (int) (calc.getProbColor() 		  * 100) );
		controlador.setProbabilidad(4, (int) (calc.getProbFullHouse() 	  * 100) );
		controlador.setProbabilidad(5, (int) (calc.getProbPoker() 		  * 100) );
		controlador.setProbabilidad(6, (int) (calc.getProbEscaleraColor() * 100) );
		controlador.setProbabilidad(7, (int) (calc.getProbEscaleraReal()  * 100) );

		controlador.setDecision( calc.getDecision().toString() );
	}
}
